use std::sync::Arc;
use server::Server;
use router::Router;

fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.add_route("GET", "/hello", |_, _| {
        (200, "hello, world".to_string())
    });

    router.add_route("GET", "/ping", |_, _| {
        (200, "pong".to_string())
    });

    let router = Arc::new(router);
    let mut server = Server::new("0.0.0.0:8082", router)?;
    server.run()?;
    
    Ok(())
}
