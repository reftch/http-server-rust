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
}

impl Response {
    pub fn new(status: u16, body: impl Into<String>, content_type: ContentType) -> Self {
        Self {
            status,
            body: body.into(),
            content_type,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let reason = match self.status {
            200 => "OK",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "",
        };

        let response = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\r\n{}",
            self.status,
            reason,
            self.content_type.as_str(), // Call the new method here
            self.body.len(),
            self.body
        );
        response.into_bytes()
    }

    pub fn set_content_type(&mut self, content_type: ContentType) {
        self.content_type = content_type;
    }
}
