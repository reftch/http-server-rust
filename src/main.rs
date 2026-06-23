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

impl Conn {
    fn new(stream: TcpStream) -> Self {
        Conn { stream }
    }

    fn read_and_respond(&mut self, buf: &mut [u8]) -> io::Result<bool> {
        loop {
            match self.stream.read(buf) {
                Ok(0) => return Ok(true),
                Ok(_) => {
                    let mut written = 0;
                    while written < RESPONSE.len() {
                        match self.stream.write(&RESPONSE[written..]) {
                            Ok(0) => return Ok(true),
                            Ok(n) => written += n,
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                            Err(_) => return Ok(true),
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => return Ok(true),
            }
        }
        Ok(false)
    }
}

pub struct Server {
    poll: Poll,
    events: Events,
    listener: TcpListener,
    conns: Slab<Conn>,
    buf: [u8; 4096],
}

impl Server {
    pub fn new(addr: SocketAddr) -> io::Result<Self> {
        let poll = Poll::new()?;
        let events = Events::with_capacity(8192);
        let mut listener = TcpListener::bind(addr)?;

        poll.registry()
            .register(&mut listener, SERVER, Interest::READABLE)?;

        let conns = Slab::with_capacity(1024);
        let buf = [0u8; 4096];

        Ok(Server {
            poll,
            events,
            listener,
            conns,
            buf,
        })
    }

    pub fn start(&mut self) -> io::Result<()> {
        loop {
            self.poll.poll(&mut self.events, None)?;

            for event in self.events.iter() {
                match event.token() {
                    SERVER => {
                        Self::handle_server_accept(
                            &mut self.listener,
                            &mut self.poll,
                            &mut self.conns,
                        )?;
                    }
                    token => {
                        Self::handle_connection_event(
                            token,
                            &mut self.poll,
                            &mut self.conns,
                            &mut self.buf,
                        )?;
                    }
                }
            }
        }
    }

    fn handle_server_accept(
        listener: &mut TcpListener,
        poll: &mut Poll,
        conns: &mut Slab<Conn>,
    ) -> io::Result<()> {
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.set_nodelay(true);

                    let entry = conns.vacant_entry();
                    let idx = entry.key();

                    poll.registry()
                        .register(&mut stream, Token(idx + 1), Interest::READABLE)?;

                    entry.insert(Conn::new(stream));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn handle_connection_event(
        token: Token,
        poll: &mut Poll,
        conns: &mut Slab<Conn>,
        buf: &mut [u8],
    ) -> io::Result<()> {
        let idx = token.0 - 1;

        if let Some(conn) = conns.get_mut(idx) {
            let close = conn.read_and_respond(buf)?;
            if close {
                if let Some(mut conn) = conns.try_remove(idx) {
                    let _ = poll.registry().deregister(&mut conn.stream);
                }
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let start_instant = Instant::now();
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let mut server = Server::new(addr)?;

    println!(
        "Server READY on http://{} | startup took {} µs",
        addr,
        start_instant.elapsed().as_micros()
    );

    server.start()
}
