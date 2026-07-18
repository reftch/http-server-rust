use std::str;

pub fn parse(buf: &[u8]) -> Option<(String, String)> {
    if !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        return None;
    }

    let request_text = match str::from_utf8(buf) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let request_line = request_text.lines().next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let raw_path = parts.next().unwrap_or("/");

    Some((method.to_string(), raw_path.to_string()))
}
