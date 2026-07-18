#[cfg(test)]
mod tests {
    use crate::Server;
    use std::sync::Arc;
    use std::net::TcpStream;
    use std::io::{Read, Write};
    use std::thread;
    use std::time::Duration;
    use http_core::{Request, Response};
    use router::{Router, Method};

    fn hello_handler(_req: &Request, res: &mut Response) {
        res.body = "Hello, World!".to_string();
    }

    #[test]
    fn test_server_connection() {
        let mut router = Router::new();
        router.add_route(Method::GET, "/", hello_handler);
        let router = Arc::new(router);

        // Use port 0 to let the OS assign a free port
        let mut server = Server::new("127.0.0.1:0", router.clone()).unwrap();
        let addr = server.listener.local_addr().unwrap();

        thread::spawn(move || {
            if let Err(e) = server.run() {
                eprintln!("Server error: {}", e);
            }
        });

        // Wait a bit for the server to start and bind
        thread::sleep(Duration::from_millis(100));

        let mut stream = TcpStream::connect(addr).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(2))).unwrap();

        // Send a basic HTTP GET request
        let request = format!("GET / HTTP/1.1\r\nHost: {}\r\n\r\n", addr.ip());
        stream.write_all(request.as_bytes()).unwrap();

        // Read the response
        let mut buffer = Vec::new();
        loop {
            let mut chunk = [0; 1024];
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    // A simple check to see if we've received the end of the response
                    if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                        // Check if there's more data after headers (the body)
                        let header_end = buffer.windows(4)
                            .position(|w| w == b"\r\n\r\n")
                            .unwrap() + 4;
                        
                        // We might need to read a bit more to get the full body if it's not immediately available
                        thread::sleep(Duration::from_millis(50));
                        let mut extra = [0; 1024];
                        if let Ok(n) = stream.read(&mut extra) {
                            buffer.extend_from_slice(&extra[..n]);
                        }
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        let response_str = String::from_utf8_lossy(&buffer);
        assert!(response_str.contains("HTTP/1.1"));
        assert!(response_str.contains("200 OK"));
        assert!(response_str.contains("Hello, World!"));
    }

    #[test]
    fn test_server_404() {
        let mut router = Router::new();
        router.add_route(Method::GET, "/", hello_handler);
        let router = Arc::new(router);

        let mut server = Server::new("127.0.0.1:0", router.clone()).unwrap();
        let addr = server.listener.local_addr().unwrap();

        thread::spawn(move || {
            let _ = server.run();
        });

        thread::sleep(Duration::from_millis(100));

        let mut stream = TcpStream::connect(addr).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(2))).unwrap();

        // Send a request for a non-existent path
        let request = format!("GET /not-found HTTP/1.1\r\nHost: {}\r\n\r\n", addr.ip());
        stream.write_all(request.as_bytes()).unwrap();

        let mut buffer = Vec::new();
        let mut chunk = [0; 1024];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        let response_str = String::from_utf8_lossy(&buffer);
        assert!(response_str.contains("404 Not Found"));
    }
}
