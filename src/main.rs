use std::sync::Arc;
use server::Server;
use router::Router;
use utils::{get_env};


fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.add_route("GET", "/hello", |_, res| {
        // println!("Handled request for path: {}", req.path);
        res.status = 200;
        res.body = "hello, world".to_string();
    });

    router.add_route("GET", "/ping", |_, res| {
        res.status = 200;
        res.body = "pong".to_string();
    });

    let router = Arc::new(router);
    let host = get_env("HOST", "0.0.0.0".to_string());
    let port = get_env("PORT", 8080);
    let addr = format!("{}:{}", host, port);

    let mut server = Server::new(&addr, router)?;
    server.run()?;
    
    Ok(())
}
