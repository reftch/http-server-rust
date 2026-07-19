#[cfg(test)]
mod tests {
    #[test]
    fn test_dummy() {}
}

use response::Status;
use router::{Method, Router};
use server::Server;
use std::sync::Arc;
use utils::get_env;

fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.add_route(Method::GET, "/api/v1/inc/:id", |req, res| {
        if let Some(id) = req.params.get("id") {
            if let Ok(val) = id.parse::<i32>() {
                res.set_content_type(response::ContentType::JSON)
                    .set_body(format!("{{\"value\":{}}}", val + 1));
            } else {
                res.set_status(Status::BadRequest)
                    .set_body("Invalid ID".to_string());
            }
        }
    });

    router.add_route(Method::GET, "/ping", |_, res| {
        res.set_body("pong".to_string());
    });

    let router = Arc::new(router);
    let host = get_env("HOST", "0.0.0.0".to_string());
    let port = get_env("PORT", 8080);
    let addr = format!("{}:{}", host, port);

    let mut server = Server::new(&addr, router)?;
    server.run()?;

    Ok(())
}
