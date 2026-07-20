use response::Status;
use router::Method;
use server::Server;
use utils::get_env;

fn main() -> std::io::Result<()> {
    let host = get_env("HOST", "0.0.0.0".to_string());
    let port = get_env("PORT", 8080);
    let addr = format!("{}:{}", host, port);

    let mut server = Server::new(&addr)?;

    server.add_route(Method::GET, "/api/v1/inc/:id", |req, res| {
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

    server.run()?;

    Ok(())
}
