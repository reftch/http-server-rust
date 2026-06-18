use mio::{
    Events, Interest, Poll, Token,
    net::{TcpListener, TcpStream},
};
use slab::Slab;

use std::{
    io::{self, Read, Write},
    net::SocketAddr,
    time::Instant,
};

const SERVER: Token = Token(0);

const RESPONSE: &[u8] = b"\
HTTP/1.1 200 OK\r\n\
Content-Length: 13\r\n\
Connection: keep-alive\r\n\
Content-Type: text/plain\r\n\
\r\n\
Hello, World!";

struct Conn {
    stream: TcpStream,
}

fn main() -> io::Result<()> {
    let start_instant = Instant::now();

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(8192);

    let mut listener = TcpListener::bind(addr)?;
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    // let mut conns: Slab<Conn> = Slab::with_capacity(65536);
    let mut conns: Slab<Conn> = Slab::with_capacity(1024);
    let mut buf = [0u8; 4096];

    println!("server starting addr: http://{}", addr);

    // small “ready” marker after bind + register
    println!(
        "server READY | startup took {} µs",
        start_instant.elapsed().as_micros()
    );

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let _ = stream.set_nodelay(true);

                            let entry = conns.vacant_entry();
                            let idx = entry.key();

                            poll.registry().register(
                                &mut stream,
                                Token(idx + 1),
                                Interest::READABLE,
                            )?;

                            entry.insert(Conn { stream });
                        }

                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(_) => break,
                    }
                },

                token => {
                    let idx = token.0 - 1;
                    let mut close = false;

                    if let Some(conn) = conns.get_mut(idx) {
                        loop {
                            match conn.stream.read(&mut buf) {
                                Ok(0) => {
                                    close = true;
                                    break;
                                }

                                Ok(_) => {
                                    let mut written = 0;

                                    while written < RESPONSE.len() {
                                        match conn.stream.write(&RESPONSE[written..]) {
                                            Ok(0) => {
                                                close = true;
                                                break;
                                            }
                                            Ok(n) => written += n,
                                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                                break;
                                            }
                                            Err(_) => {
                                                close = true;
                                                break;
                                            }
                                        }
                                    }
                                }

                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    break;
                                }

                                Err(_) => {
                                    close = true;
                                    break;
                                }
                            }
                        }
                    }

                    if close {
                        if let Some(mut conn) = conns.try_remove(idx) {
                            let _ = poll.registry().deregister(&mut conn.stream);
                        }
                    }
                }
            }
        }
    }
}
