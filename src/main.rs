use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;

const DEFAULT_ADDR: &str = "127.0.0.1:8082";
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

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(DEFAULT_ADDR.parse::<std::net::SocketAddr>().unwrap())?;
    listener.set_nonblocking(true)?;

    let mut poll_fds: Vec<PollFd> = vec![PollFd {
        fd: listener.as_raw_fd(),
        events: POLLIN,
        revents: 0,
    }];

    let mut connections: HashMap<usize, Connection> = HashMap::new();

    println!("Listening on http://{}", listener.local_addr()?);

    loop {
        for pfd in poll_fds.iter_mut() {
            pfd.revents = 0;
        }

        let nfds = unsafe {
            libc::poll(
                poll_fds.as_mut_ptr() as *mut libc::pollfd,
                poll_fds.len() as libc::nfds_t,
                1000,
            )
        };

        println!("Connections size {}", connections.len());
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
                match listener.accept() {
                    Ok((stream, _)) => {
                        let conn = Connection::new(stream)?;
                        let idx = poll_fds.len();
                        let fd = conn.socket.as_raw_fd();
                        poll_fds.push(PollFd {
                            fd,
                            events: POLLIN,
                            revents: 0,
                        });
                        connections.insert(idx, conn);
                    }
                    Err(ref err) if would_block(err) => break,
                    Err(err) => {
                        eprintln!("Accept error: {}", err);
                        break;
                    }
                }
            }
        }

        // Handle client connections
        let mut indices_to_remove = Vec::new();

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
                if let Some(conn) = connections.get_mut(&i) {
                    match handle_write(conn) {
                        Ok(true) => {
                            poll_fds[i].events = POLLIN;
                        }
                        Ok(false) => {
                            indices_to_remove.push(i);
                        }
                        Err(err) => {
                            eprintln!("Write error: {}", err);
                            indices_to_remove.push(i);
                        }
                    }
                }
            } else if revents & POLLIN != 0 {
                if let Some(conn) = connections.get_mut(&i) {
                    match handle_read(conn) {
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
            connections.remove(i);
            poll_fds.remove(*i);
        }
    }
}

fn handle_read(conn: &mut Connection) -> io::Result<bool> {
    let mut buf = [0; 1024];
    loop {
        match conn.socket.read(&mut buf) {
            Ok(0) => return Ok(false),
            Ok(n) => conn.read_buf.extend_from_slice(&buf[..n]),
            Err(ref err) if would_block(err) => break,
            Err(err) => return Err(err),
        }
    }

    if !conn.read_buf.windows(4).any(|w| w == b"\r\n\r\n") {
        return Ok(true);
    }

    let request_text = String::from_utf8_lossy(&conn.read_buf).into_owned();
    let request_line = request_text.lines().next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let raw_path = parts.next().unwrap_or("/");

    conn.read_buf.clear();
    conn.write_buf = if method == "GET" && raw_path == "/hello" {
        text_response(200, "hello, world")
    } else {
         text_response(404, "Not found")
    };

    Ok(true)
}

fn handle_write(conn: &mut Connection) -> io::Result<bool> {
    while !conn.write_buf.is_empty() {
        match conn.socket.write(&conn.write_buf) {
            Ok(0) => return Ok(false),
            Ok(n) => {
                conn.write_buf.drain(0..n);
            }
            Err(ref err) if would_block(err) => return Ok(true),
            Err(err) => return Err(err),
        }
    }
    Ok(true)
}

fn would_block(err: &io::Error) -> bool {
    matches!(err.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted)
}

fn text_response(status: u16, body: &str) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{}",
        status,
        reason,
        body.len(),
        body
    );
    response.into_bytes()
}
