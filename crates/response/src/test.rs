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
    fn test_content_type_as_str() {
        assert_eq!(crate::ContentType::HTML.as_str(), "text/html");
        assert_eq!(crate::ContentType::JSON.as_str(), "application/json");
        assert_eq!(crate::ContentType::UNKNOWN.as_str(), "application/octet-stream");
    }

    #[test]
    fn test_response_add_header() {
        let mut response = Response::new(200, "OK", crate::ContentType::TEXT);
        response.add_header("X-Test".to_string(), "Value".to_string());
        assert_eq!(response.headers.get("X-Test").unwrap(), "Value");

        // Ensure duplicate headers are not added (as per implementation)
        response.add_header("X-Test".to_string(), "New Value".to_string());
        assert_eq!(response.headers.get("X-Test").unwrap(), "Value");
    }

    #[test]
    fn test_response_set_content_type() {
        let mut response = Response::new(200, "OK", crate::ContentType::TEXT);
        response.set_content_type(crate::ContentType::JSON);
        assert_eq!(response.content_type, crate::ContentType::JSON);
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

    #[test]
    fn test_response_to_bytes_with_headers() {
        let mut response = Response::new(200, "OK", crate::ContentType::TEXT);
        response.add_header("Custom-Header".to_string(), "Custom-Value".to_string());
        let bytes = response.to_bytes();
        let bytes_str = String::from_utf8(bytes).unwrap();
        assert!(bytes_str.contains("Custom-Header: Custom-Value\r\n"));
    }
