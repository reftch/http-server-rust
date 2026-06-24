mod request;
mod response;
mod server;

use server::Server;

fn main() -> std::io::Result<()> {
    let mut server = Server::new("0.0.0.0:8082")?;
    server.run()?;
    
    Ok(())
}
