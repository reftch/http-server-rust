use crate::Response;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_new() {
        let response = Response::new(200, "Hello World", crate::ContentType::TEXT);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "Hello World");
        assert_eq!(response.content_type, crate::ContentType::TEXT);
    }

    #[test]
    fn test_response_to_bytes() {
        let response = Response::new(200, "OK", crate::ContentType::TEXT);
        let bytes = response.to_bytes();
        let bytes_str = String::from_utf8(bytes).unwrap();
        assert!(bytes_str.contains("HTTP/1.1 200 OK"));
        assert!(bytes_str.contains("Content-Length: 2"));
        assert!(bytes_str.ends_with("OK"));
        assert_eq!(response.content_type, crate::ContentType::TEXT);
    }

    #[test]
    fn test_response_404() {
        let response = Response::new(404, "Not Found", crate::ContentType::TEXT);
        let bytes = response.to_bytes();
        let bytes_str = String::from_utf8(bytes).unwrap();
        assert!(bytes_str.contains("HTTP/1.1 404 Not Found"));
        assert_eq!(response.content_type, crate::ContentType::TEXT);
    }
}
