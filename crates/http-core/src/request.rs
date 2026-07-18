use std::collections::HashMap;

pub struct Request {
    pub method: String,
    pub path: String,
    pub params: HashMap<String, String>,
}

impl Request {
    pub fn parse(buf: &[u8]) -> Option<Self> {
        use std::str;

        if !buf.windows(4).any(|w| w == b"\r\n\r\n") {
            return None;
        }

        let request_text = match str::from_utf8(buf) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let request_line = request_text.lines().next().unwrap_or("");
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let path = parts.next().unwrap_or("/").to_string();

        Some(Request { 
            method, 
            path,
            params: HashMap::new(),
        })
    }
}
