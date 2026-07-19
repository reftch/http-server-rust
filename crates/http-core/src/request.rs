use std::collections::HashMap;

pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub headers: HashMap<&'a str, &'a str>,
    pub params: HashMap<String, String>,
}

impl<'a> Request<'a> {
    #[inline]
    fn find_header_end(buf: &[u8]) -> Option<usize> {
        let mut i = 0;

        while i + 3 < buf.len() {
            if buf[i] == b'\r' && buf[i + 1] == b'\n' && buf[i + 2] == b'\r' && buf[i + 3] == b'\n'
            {
                return Some(i + 4);
            }

            i += 1;
        }

        None
    }

    pub fn parse(buf: &'a [u8]) -> Option<Self> {
        let header_end = Self::find_header_end(buf)?;

        let text = std::str::from_utf8(&buf[..header_end]).ok()?;

        let mut lines = text.split_terminator("\r\n");

        // Request line
        let request_line = lines.next()?;

        let first_space = request_line.find(' ')?;
        let second_space = match request_line[first_space + 1..].find(' ') {
            Some(i) => i + first_space + 1,
            None => return None,
        };

        if second_space >= request_line.len().saturating_sub(1) {
            return None;
        }

        let method = &request_line[..first_space];
        let path = &request_line[first_space + 1..second_space];

        if method.is_empty() || path.is_empty() {
            return None;
        }

        // Headers
        let mut headers = HashMap::with_capacity(12);

        for line in lines {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };

            headers.insert(key.trim(), value.trim());
        }

        Some(Self {
            method,
            path,
            headers,
            params: HashMap::with_capacity(4),
        })
    }

    #[inline]
    pub fn mime_type(&self) -> Option<&str> {
        self.headers.get("Content-Type").copied()
    }
}
