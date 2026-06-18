use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use slab::Slab;
use socket2::{Domain, Socket, Type};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::time::Duration;
use std::thread;
use num_cpus;

const SERVER: Token = Token(0);

struct Connection {
    socket: TcpStream,
    read_buf: Vec<u8>,
    write_buf: &'static [u8],
    write_pos: usize,
    keep_alive: bool,
}

// Precomputed constant response to avoid per-request allocation/formatting.
const RESPONSE_CLOSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 14\r\nConnection: close\r\n\r\nHello, World!\n";
const RESPONSE_KEEPALIVE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 14\r\nConnection: keep-alive\r\n\r\nHello, World!\n";

fn worker_loop(std_listener: std::net::TcpListener) -> std::io::Result<()> {
    let std_listener = std_listener;
    std_listener.set_nonblocking(true)?;
    let mut listener = TcpListener::from_std(std_listener);

    let mut poll = Poll::new()?;
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    let mut events = Events::with_capacity(1024);
    let mut slab: Slab<Connection> = Slab::with_capacity(4096);

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;

        for event in &events {
            match event.token() {
                SERVER => loop {
                    match listener.accept() {
                        Ok((socket, _addr)) => {
                            socket.set_nodelay(true)?;
                            let entry = slab.vacant_entry();
                            let key = entry.key();
                            entry.insert(Connection {
                                socket,
                                read_buf: Vec::with_capacity(1024),
                                write_buf: &[],
                                write_pos: 0,
                                keep_alive: false,
                            });
                            let token = Token(key + 1); // reserve 0 for server
                            let conn = &mut slab[token.0 - 1];
                            poll.registry().register(
                                &mut conn.socket,
                                token,
                                Interest::READABLE.add(Interest::WRITABLE),
                            )?;
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            eprintln!("accept error: {e}");
                            break;
                        }
                    }
                },
                t => {
                    let idx = t.0 - 1;
                    if slab.get(idx).is_none() {
                        continue;
                    }
                    let mut done = false;
                    {
                        let conn = &mut slab[idx];
                        if event.is_readable() {
                            let mut buf = [0u8; 1024];
                            loop {
                                match conn.socket.read(&mut buf) {
                                    Ok(0) => {
                                        done = true;
                                        break;
                                    }
                                    Ok(n) => {
                                        conn.read_buf.extend_from_slice(&buf[..n]);
                                        // check for end of HTTP headers
                                        if let Some(pos) = find_header_end(&conn.read_buf) {
                                            // simple header parsing: look for Connection header or HTTP version
                                            let headers = &conn.read_buf[..pos];
                                            let headers_lc = to_lowercase_bytes(headers);
                                            // let has_conn_close = headers_lc.windows(14).any(|w| w == b"connection: ");
                                            // determine keep-alive: prefer explicit header, else HTTP/1.1 defaults to keep-alive
                                            let keep = if headers_lc.windows(17).any(|w| w == b"connection: close") {
                                                false
                                            } else if headers_lc.windows(21).any(|w| w == b"connection: keep-alive") {
                                                true
                                            } else if headers_lc.windows(8).any(|w| w == b"http/1.1") {
                                                true
                                            } else {
                                                false
                                            };
                                            conn.keep_alive = keep;
                                            conn.write_buf = if keep { RESPONSE_KEEPALIVE } else { RESPONSE_CLOSE };
                                            conn.write_pos = 0;
                                            // drop request bytes (not handling pipelining for now)
                                            conn.read_buf.clear();
                                            break;
                                        }
                                    }
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                                    Err(e) => {
                                        eprintln!("read error: {e}");
                                        done = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if event.is_writable() && !conn.write_buf.is_empty() {
                            while conn.write_pos < conn.write_buf.len() {
                                match conn
                                    .socket
                                    .write(&conn.write_buf[conn.write_pos..])
                                {
                                    Ok(0) => {
                                        done = true;
                                        break;
                                    }
                                    Ok(n) => conn.write_pos += n,
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                                    Err(e) => {
                                        eprintln!("write error: {e}");
                                        done = true;
                                        break;
                                    }
                                }
                            }
                            if conn.write_pos >= conn.write_buf.len() {
                                if conn.keep_alive {
                                    // reset for next request
                                    conn.write_buf = &[];
                                    conn.write_pos = 0;
                                } else {
                                    done = true; // close after sending response
                                }
                            }
                        }
                    }

                    if done {
                        // deregister and drop
                        let mut conn = slab.remove(idx);
                        let _ = poll.registry().deregister(&mut conn.socket);
                    }
                }
            }
        }
    }
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    // Look for CRLF CRLF or LF LF
    for i in 0..buf.len().saturating_sub(3) {
        if &buf[i..i + 4] == b"\r\n\r\n" {
            return Some(i + 4);
        }
    }
    for i in 0..buf.len().saturating_sub(1) {
        if &buf[i..i + 2] == b"\n\n" {
            return Some(i + 2);
        }
    }
    None
}

fn to_lowercase_bytes(s: &[u8]) -> Vec<u8> {
    s.iter().map(|b| b.to_ascii_lowercase()).collect()
}

fn main() -> std::io::Result<()> {
    let addr: SocketAddr = "127.0.0.1:8082".parse().unwrap();
    let workers = num_cpus::get();
    println!("Starting {} worker threads on http://{}", workers, addr);

    // Create the listening socket once and clone it for each worker.
    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(1024)?;
    let std_listener: std::net::TcpListener = socket.into();

    let mut handles = Vec::with_capacity(workers);
    for _ in 0..workers {
        let listener_clone = std_listener.try_clone()?;
        handles.push(thread::spawn(move || {
            if let Err(e) = worker_loop(listener_clone) {
                eprintln!("worker error: {e}");
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    Ok(())
}