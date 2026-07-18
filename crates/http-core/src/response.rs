pub fn send(status: u16, body: &str) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        reason,
        body.len(),
        body
    );
    response.into_bytes()
}
