pub mod server;

use server::server::SecretsServer;

fn main() -> std::io::Result<()> {
    let server = SecretsServer::new();
    server.run()
}
