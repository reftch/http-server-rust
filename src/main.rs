mod request;
mod response;
mod server;

use server::Server;

fn main() -> std::io::Result<()> {
    let mut server = Server::new("127.0.0.1:8082")?;
    server.run()?;
    
    Ok(())
}
