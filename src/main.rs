mod request;
mod response;
mod server;

fn main() -> std::io::Result<()> {
    server::run()
}
