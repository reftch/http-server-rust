#[cfg(test)]
mod tests {
    #[test]
    fn test_dummy() {}
}

use router::{Method, Router};
use server::Server;
use std::sync::Arc;
use utils::get_env;

fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.add_route(Method::GET, "/api/:version/inc/:id", |req, res| {
        if let Some(id) = req.params.get("id") {
            if let Ok(val) = id.parse::<i32>() {
                res.status = 200;
                res.body = (val + 1).to_string();
            } else {
                res.status = 400;
                res.body = "Invalid ID".to_string();
            }
        }
    });

    router.add_route(Method::GET, "/ping", |_, res| {
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
