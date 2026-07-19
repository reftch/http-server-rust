#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use std::time::Instant;

use request::Request;
use response::Response;
use router::Router;

use std::sync::Arc;

const POLLIN: i16 = 0x001;
const POLLOUT: i16 = 0x004;
const POLLERR: i16 = 0x008;
const POLLHUP: i16 = 0x010;

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

struct Connection {
    socket: std::net::TcpStream,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
}

impl Connection {
    fn new(socket: std::net::TcpStream) -> io::Result<Connection> {
        socket.set_nonblocking(true)?;
        Ok(Connection {
            socket,
            read_buf: Vec::with_capacity(1024),
            write_buf: Vec::new(),
        })
    }
}

enum WriteState {
    Continue,
    Done,
    Close,
}

pub struct Server {
    init_start: Instant,
    listener: TcpListener,
    router: Arc<Router>,
}

impl Server {
    pub fn new(addr: &str, router: Arc<Router>) -> io::Result<Self> {
        Ok(Server {
            init_start: Instant::now(),
            listener: TcpListener::bind(addr.parse::<std::net::SocketAddr>().unwrap())?,
            router,
        })
    }

    fn would_block(err: &io::Error) -> bool {
        matches!(
            err.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
        )
    }

    fn handle_write(conn: &mut Connection) -> io::Result<WriteState> {
        loop {
            if conn.write_buf.is_empty() {
                return Ok(WriteState::Done);
            }

            match conn.socket.write(&conn.write_buf) {
                Ok(0) => return Ok(WriteState::Close),
                Ok(n) => {
                    conn.write_buf.drain(0..n);
                }
                Err(ref err) if Self::would_block(err) => return Ok(WriteState::Continue),
                Err(err) => return Err(err),
            }
        }
    }

    fn handle_read(conn: &mut Connection, router: &Router) -> io::Result<bool> {
        let mut buf = [0; 1024];
        loop {
            match conn.socket.read(&mut buf) {
                Ok(0) => return Ok(false),
                Ok(n) => conn.read_buf.extend_from_slice(&buf[..n]),
                Err(ref err) if Self::would_block(err) => break,
                Err(err) => return Err(err),
            }
        }

        if let Some(mut request) = Request::parse(&conn.read_buf) {
            let response = if let Some(resp) = router.route(&mut request) {
                resp
            } else {
                     Response::new(
                         404,
                         "Not found".to_string(),
                         response::ContentType::TEXT,
                     )
            };

            conn.write_buf = response.to_bytes();
            conn.read_buf.clear();
        }

        Ok(true)
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.listener.set_nonblocking(true)?;

        let mut poll_fds: Vec<PollFd> = vec![PollFd {
            fd: self.listener.as_raw_fd(),
            events: POLLIN,
            revents: 0,
        }];

        let mut connections: HashMap<i32, Connection> = HashMap::new();

        let startup_us = self.init_start.elapsed().as_micros();
        println!(
            "Server started on http://{} in {}µs",
            self.listener.local_addr()?,
            startup_us
        );

        let mut indices_to_remove = Vec::new();

        loop {
            for pfd in poll_fds.iter_mut() {
                pfd.revents = 0;
            }

            let nfds = unsafe {
                libc::poll(
                    poll_fds.as_mut_ptr() as *mut libc::pollfd,
                    poll_fds.len() as libc::nfds_t,
                    2000,
                )
            };

            if nfds < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(err);
            }

            if nfds == 0 {
                continue;
            }

            // Handle listener first (index 0)
            if poll_fds[0].revents & POLLIN != 0 {
                loop {
                    match self.listener.accept() {
                        Ok((stream, _)) => {
                            let conn = Connection::new(stream)?;
                            let fd = conn.socket.as_raw_fd();
                            poll_fds.push(PollFd {
                                fd,
                                events: POLLIN,
                                revents: 0,
                            });
                            connections.insert(fd, conn);
                        }
                        Err(ref err) if Self::would_block(err) => break,
                        Err(err) => {
                            eprintln!("Accept error: {}", err);
                            break;
                        }
                    }
                }
            }

            // Handle client connections
            indices_to_remove.clear();

            for i in 1..poll_fds.len() {
                if poll_fds[i].revents == 0 {
                    continue;
                }

                let revents = poll_fds[i].revents;

                if revents & (POLLERR | POLLHUP) != 0 {
                    indices_to_remove.push(i);
                    continue;
                }

                if revents & POLLOUT != 0 {
                    let fd = poll_fds[i].fd;
                    if let Some(conn) = connections.get_mut(&fd) {
                        match Self::handle_write(conn) {
                            Ok(WriteState::Done) => {
                                poll_fds[i].events = POLLIN;
                            }
                            Ok(WriteState::Continue) => {
                                // still have data to write; keep POLLOUT
                            }
                            Ok(WriteState::Close) => {
                                indices_to_remove.push(i);
                            }
                            Err(err) => {
                                eprintln!("Write error: {}", err);
                                indices_to_remove.push(i);
                            }
                        }
                    }
                } else if revents & POLLIN != 0 {
                    let fd = poll_fds[i].fd;
                    if let Some(conn) = connections.get_mut(&fd) {
                        match Self::handle_read(conn, &self.router) {
                            Ok(true) => {
                                if !conn.write_buf.is_empty() {
                                    poll_fds[i].events = POLLOUT;
                                }
                            }
                            Ok(false) => {
                                indices_to_remove.push(i);
                            }
                            Err(err) => {
                                eprintln!("Read error: {}", err);
                                indices_to_remove.push(i);
                            }
                        }
                    }
                }
            }

            for i in indices_to_remove.iter().rev() {
                let fd = poll_fds[*i].fd;
                connections.remove(&fd);
                poll_fds.remove(*i);
            }
        }
    }
}
