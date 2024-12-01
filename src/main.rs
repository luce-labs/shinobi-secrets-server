pub mod server;
pub mod types;

use server::server::SecretsServer;

fn main() -> std::io::Result<()> {
    let server = SecretsServer::new();
    server.run()
}
