use super::*;
use http_core::{Request, Response};
use std::collections::HashMap;

fn hello_handler(_req: &Request, res: &mut Response) {
    res.body = "Hello, World!".to_string();
}

fn param_handler(req: &Request, res: &mut Response) {
    let name = req.params.get("name").unwrap().clone();
    res.body = format!("Hello, {}!", name);
}

#[test]
fn test_add_and_route_basic() {
    let mut router = Router::new();
    router.add_route(Method::GET, "/", hello_handler);
    let mut req = Request {
        method: "GET".to_string(),
        path: "/".to_string(),
        params: HashMap::new(),
    };

    let res = router.route(&mut req).expect("Route should be found");
    assert_eq!(res.status, 200);
    assert_eq!(res.body, "Hello, World!");
}

#[test]
fn test_route_not_found() {
    let mut router = Router::new();
    router.add_route(Method::GET, "/", hello_handler);

    let mut req = Request {
        method: "GET".to_string(),
        path: "/not-found".to_string(),
        params: HashMap::new(),
    };

    assert!(router.route(&mut req).is_none());
}

#[test]
fn test_route_with_params() {
    let mut router = Router::new();
    router.add_route(Method::GET, "/user/:name", param_handler);

    let mut req = Request {
        method: "GET".to_string(),
        path: "/user/alice".to_string(),
        params: HashMap::new(),
    };

    let res = router.route(&mut req).expect("Route should be found");
    assert_eq!(res.status, 200);
    assert_eq!(res.body, "Hello, alice!");
    assert_eq!(req.params.get("name").unwrap(), "alice");
}

#[test]
fn test_different_methods() {
    let mut router = Router::new();
    router.add_route(Method::GET, "/path", hello_handler);
    router.add_route(Method::POST, "/path", |_, res| {
        res.body = "POST handled".to_string();
    });

    let mut req_get = Request {
        method: "GET".to_string(),
        path: "/path".to_string(),
        params: HashMap::new(),
    };
    let res_get = router.route(&mut req_get).unwrap();
    assert_eq!(res_get.body, "Hello, World!");

    let mut req_post = Request {
        method: "POST".to_string(),
        path: "/path".to_string(),
        params: HashMap::new(),
    };
    let res_post = router.route(&mut req_post).unwrap();
    assert_eq!(res_post.body, "POST handled");
}

#[test]
fn test_method_from_str() {
    assert_eq!(Method::from_str("GET"), Some(Method::GET));
    assert_eq!(Method::from_str("POST"), Some(Method::POST));
    assert_eq!(Method::from_str("INVALID"), None);
}

#[test]
fn test_method_index() {
    assert_eq!(Method::GET.index(), 0);
    assert_eq!(Method::OPTIONS.index(), 6);
}
