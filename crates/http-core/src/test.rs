use crate::request::Request;
use crate::response::Response;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_parse_valid() {
        let buf = b"GET / HTTP/1.1\r\n\r\n";
        let request = Request::parse(buf).expect("Should parse valid request");
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/");
    }

    #[test]
    fn test_request_parse_invalid() {
        let buf = b"GET / HTTP/1.1\r\n";
        let request = Request::parse(buf);
        assert!(request.is_none());
    }

    #[test]
    fn test_response_new() {
        let response = Response::new(200, "Hello World");
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "Hello World");
    }

    #[test]
    fn test_response_to_bytes() {
        let response = Response::new(200, "OK");
        let bytes = response.to_bytes();
        let bytes_str = String::from_utf8(bytes).unwrap();
        assert!(bytes_str.contains("HTTP/1.1 200 OK"));
        assert!(bytes_str.contains("Content-Length: 2"));
        assert!(bytes_str.ends_with("OK"));
    }

    #[test]
    fn test_response_404() {
        let response = Response::new(404, "Not Found");
        let bytes = response.to_bytes();
        let bytes_str = String::from_utf8(bytes).unwrap();
        assert!(bytes_str.contains("HTTP/1.1 404 Not Found"));
    }
}
