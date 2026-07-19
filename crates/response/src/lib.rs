use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ContentType {
    HTML,
    CSS,
    JAVASCRIPT,
    JPEG,
    PNG,
    XML,
    JSON,
    TEXT,
    GIF,
    SVG,
    PDF,
    MP3,
    MP4,
    WEBM,
    WOFF2,
    TTF,
    EOT,
    SSE, // Server-Sent Events
    UNKNOWN,
}

impl ContentType {
    /// Returns the standard MIME type string for the content type.
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::HTML => "text/html",
            ContentType::CSS => "text/css",
            ContentType::JAVASCRIPT => "text/javascript",
            ContentType::JPEG => "image/jpeg",
            ContentType::PNG => "image/png",
            ContentType::XML => "application/xml",
            ContentType::JSON => "application/json",
            ContentType::TEXT => "text/plain",
            ContentType::GIF => "image/gif",
            ContentType::SVG => "image/svg+xml",
            ContentType::PDF => "application/pdf",
            ContentType::MP3 => "audio/mpeg",
            ContentType::MP4 => "video/mp4",
            ContentType::WEBM => "video/webm",
            ContentType::WOFF2 => "font/woff2",
            ContentType::TTF => "font/ttf",
            ContentType::EOT => "application/vnd.ms-fontobject",
            ContentType::SSE => "text/event-stream",
            ContentType::UNKNOWN => "application/octet-stream",
        }
    }
}

pub struct Response {
    pub status: u16,
    pub body: String,
    pub content_type: ContentType,
    pub headers: HashMap<String, String>,
}

impl Response {
    pub fn new(status: u16, body: impl Into<String>, content_type: ContentType) -> Self {
        Self {
            status,
            body: body.into(),
            content_type,
            headers: HashMap::new(),
        }
    }

    pub fn add_header(&mut self, key: String, value: String) {
        if !self.headers.contains_key(&key) {
            self.headers.insert(key, value);
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let reason = match self.status {
            200 => "OK",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "",
        };

        let mut response = format!("HTTP/1.1 {} {}\r\n", self.status, reason);

        // Add Content-Type header
        response.push_str(&format!("Content-Type: {}\r\n", self.content_type.as_str()));

        // Add Content-Length header
        response.push_str(&format!("Content-Length: {}\r\n", self.body.len()));

        // Add custom headers from the collection
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        response.push_str("\r\n");
        response.push_str(&self.body);
        response.into_bytes()
    }

    pub fn set_content_type(&mut self, content_type: ContentType) {
        self.content_type = content_type;
    }
}

#[cfg(test)]
mod test;
