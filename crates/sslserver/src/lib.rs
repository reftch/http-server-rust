use libc::{POLLERR, POLLHUP, POLLIN, POLLOUT};
use openssl::ssl::{
    HandshakeError, MidHandshakeSslStream, SslAcceptor, SslFiletype, SslMethod, SslStream,
};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::Instant;

use request::Request;
use response::{ContentType, Response, Status};
use router::Router;

use std::sync::Arc;

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

enum TlsState {
    Handshaking(MidHandshakeSslStream<TcpStream>),
    Connected(SslStream<TcpStream>),
}

struct Connection {
    tls: Option<TlsState>,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
}

impl Connection {
    fn new(tls: TlsState) -> Self {
        Self {
            tls: Some(tls),
            read_buf: Vec::with_capacity(1024),
            write_buf: Vec::new(),
        }
    }

    fn fd(&self) -> i32 {
        match self.tls.as_ref().unwrap() {
            TlsState::Connected(s) => s.get_ref().as_raw_fd(),
            TlsState::Handshaking(s) => s.get_ref().as_raw_fd(),
        }
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
    assets_path: PathBuf,
    acceptor: Arc<SslAcceptor>,
}

impl Server {
    pub fn new(addr: &str) -> io::Result<Self> {
        Self::new_with_assets(addr, PathBuf::from("./assets"))
    }

    fn new_with_assets(addr: &str, assets_path: PathBuf) -> io::Result<Self> {
        let router = Arc::new(Router::new());
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

        builder
            .set_private_key_file("key.pem", SslFiletype::PEM)
            .unwrap();

        builder.set_certificate_chain_file("cert.pem").unwrap();

        Ok(Server {
            init_start: Instant::now(),
            listener: TcpListener::bind(addr.parse::<std::net::SocketAddr>().unwrap())?,
            router,
            assets_path,
            acceptor: Arc::new(builder.build()),
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

            match conn.tls.as_mut().unwrap() {
                TlsState::Handshaking(_) => {
                    // TLS is not ready yet
                    return Ok(WriteState::Continue);
                }

                TlsState::Connected(stream) => match stream.write(&conn.write_buf) {
                    Ok(0) => {
                        return Ok(WriteState::Close);
                    }
                    Ok(n) => {
                        conn.write_buf.drain(0..n);
                    }
                    Err(ref err) if Self::would_block(err) => {
                        return Ok(WriteState::Continue);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                },
            }
        }
    }

    fn handle_read(conn: &mut Connection, router: &Router, assets_path: &Path) -> io::Result<bool> {
        let mut buf = [0u8; 1024];

        loop {
            let stream = match conn.tls.as_mut().unwrap() {
                TlsState::Connected(stream) => stream,

                // TLS handshake is not complete yet
                TlsState::Handshaking(_) => {
                    return Ok(true);
                }
            };

            match stream.read(&mut buf) {
                Ok(0) => {
                    return Ok(false);
                }
                Ok(n) => {
                    conn.read_buf.extend_from_slice(&buf[..n]);
                }
                Err(ref err) if Self::would_block(err) => {
                    break;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        if let Some(mut request) = Request::parse(&conn.read_buf) {
            let response = if let Some(resp) = router.route(&mut request) {
                resp
            } else if let Some(resp) = Self::handle_static(request.path, assets_path) {
                resp
            } else {
                Response::new(Status::NotFound, "Not Found", ContentType::TEXT)
            };

            conn.write_buf = response.to_bytes();
            conn.read_buf.clear();
        }

        Ok(true)
    }

    fn handle_static(path: &str, assets_path: &Path) -> Option<Response> {
        let requested_path = Path::new(path);

        // Prevent directory traversal
        if requested_path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return None;
        }

        let mut full_path =
            assets_path.join(requested_path.strip_prefix("/").unwrap_or(requested_path));

        if full_path.is_dir() {
            full_path.push("index.html");
        }

        if !full_path.is_file() {
            return None;
        }

        let content = fs::read(&full_path).ok()?;
        let content_type = Self::get_content_type(&full_path);

        Some(Response {
            status: Status::Ok,
            body: content,
            content_type,
            headers: HashMap::new(),
        })
    }

    fn get_content_type(path: &Path) -> ContentType {
        match path.extension().and_then(|s| s.to_str()) {
            Some("html") => ContentType::HTML,
            Some("css") => ContentType::CSS,
            Some("js") => ContentType::JAVASCRIPT,
            Some("jpg") | Some("jpeg") => ContentType::JPEG,
            Some("png") => ContentType::PNG,
            Some("xml") => ContentType::XML,
            Some("json") => ContentType::JSON,
            Some("txt") => ContentType::TEXT,
            Some("gif") => ContentType::GIF,
            Some("svg") => ContentType::SVG,
            Some("pdf") => ContentType::PDF,
            Some("mp3") => ContentType::MP3,
            Some("mp4") => ContentType::MP4,
            Some("webm") => ContentType::WEBM,
            Some("woff2") => ContentType::WOFF2,
            Some("ttf") => ContentType::TTF,
            _ => ContentType::UNKNOWN,
        }
    }

    pub fn set_assets_path(&mut self, path: &str) {
        self.assets_path = PathBuf::from(path);
    }

    pub fn add_route(&mut self, method: router::Method, path: &str, handler: router::HandlerFn) {
        if let Some(router) = std::sync::Arc::get_mut(&mut self.router) {
            router.add_route(method, path, handler);
        }
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
            "HTTPS server started on https://{} in {}µs",
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

            //
            // Accept HTTPS clients
            //
            if poll_fds[0].revents & POLLIN != 0 {
                loop {
                    match self.listener.accept() {
                        Ok((stream, _addr)) => {
                            stream.set_nonblocking(true)?;

                            let tls_state = match self.acceptor.accept(stream) {
                                Ok(ssl) => TlsState::Connected(ssl),
                                Err(HandshakeError::WouldBlock(mid)) => TlsState::Handshaking(mid),
                                Err(_) => {
                                    // eprintln!("TLS handshake failed: {:?}", err);
                                    continue;
                                }
                            };

                            let conn = Connection::new(tls_state);

                            let fd = conn.fd();

                            poll_fds.push(PollFd {
                                fd,
                                // handshake needs both directions
                                events: POLLIN | POLLOUT,
                                revents: 0,
                            });

                            connections.insert(fd, conn);
                        }

                        Err(ref err) if Self::would_block(err) => {
                            break;
                        }

                        Err(err) => {
                            eprintln!("Accept error: {}", err);
                            break;
                        }
                    }
                }
            }

            indices_to_remove.clear();

            //
            // Client connections
            //
            for (i, item) in poll_fds.iter_mut().enumerate().skip(1) {
                if item.revents == 0 {
                    continue;
                }

                let fd = item.fd; // No need for poll_fds[i].fd
                let events = item.revents;

                if events & (POLLERR | POLLHUP) != 0 {
                    indices_to_remove.push(i);
                    continue;
                }

                if let Some(conn) = connections.get_mut(&fd) {
                    // Finish TLS handshake
                    if matches!(conn.tls.as_ref(), Some(TlsState::Handshaking(_))) {
                        match Self::continue_handshake(conn) {
                            Ok(true) => {
                                item.events = POLLIN; // Use 'item' instead of 'poll_fds[i]'
                            }
                            Ok(false) => {
                                item.events = POLLIN | POLLOUT;
                                continue;
                            }
                            Err(_) => {
                                indices_to_remove.push(i);
                                continue;
                            }
                        }
                    }

                    // Write HTTPS response
                    if events & POLLOUT != 0 {
                        match Self::handle_write(conn) {
                            Ok(WriteState::Done) => {
                                item.events = POLLIN;
                            }
                            Ok(WriteState::Continue) => {
                                item.events = POLLOUT;
                            }
                            Ok(WriteState::Close) => {
                                indices_to_remove.push(i);
                            }
                            Err(_) => {
                                indices_to_remove.push(i);
                            }
                        }
                    }

                    // Read HTTPS request
                    if events & POLLIN != 0 {
                        match Self::handle_read(conn, &self.router, &self.assets_path) {
                            Ok(true) => {
                                if !conn.write_buf.is_empty() {
                                    item.events = POLLOUT;
                                }
                            }
                            Ok(false) => {
                                indices_to_remove.push(i);
                            }
                            Err(_) => {
                                indices_to_remove.push(i);
                            }
                        }
                    }
                }
            }

            //
            // Remove closed connections
            //
            for i in indices_to_remove.iter().rev() {
                let fd = poll_fds[*i].fd;
                connections.remove(&fd);
                poll_fds.remove(*i);
            }
        }
    }

    fn continue_handshake(conn: &mut Connection) -> io::Result<bool> {
        let state = conn.tls.take().unwrap();

        match state {
            TlsState::Connected(stream) => {
                conn.tls = Some(TlsState::Connected(stream));
                Ok(true)
            }

            TlsState::Handshaking(mid) => match mid.handshake() {
                Ok(stream) => {
                    conn.tls = Some(TlsState::Connected(stream));
                    Ok(true)
                }
                Err(HandshakeError::WouldBlock(mid)) => {
                    conn.tls = Some(TlsState::Handshaking(mid));
                    Ok(false)
                }
                Err(e) => Err(io::Error::other(format!("{:?}", e))),
            },
        }
    }
}
