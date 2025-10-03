use redis_starter_rust::run_server;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<String>>();

    // Parse port from command line arguments
    let port = args
        .iter()
        .position(|arg| arg == "--port")
        .and_then(|idx| args.get(idx + 1))
        .and_then(|port_str| port_str.parse::<u16>().ok())
        .unwrap_or(6379);

    run_server(port, args).await
}
