#![allow(dead_code)]
use anyhow::anyhow;
use redis_starter_rust::run_server;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

pub struct TestServer {
    handle: JoinHandle<()>,
    pub addr: SocketAddr,
}

impl TestServer {
    /// Start the Redis server for testing
    pub async fn start() -> anyhow::Result<Self> {
        let port = portpicker::pick_unused_port().expect("No ports available");
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        let handle = tokio::spawn(async move {
            if let Err(e) = run_server(port, vec![]).await {
                eprintln!("Failed to start server: {:?}", e);
            }
        });
        Self::ensure_started(addr).await.map(|dur| {
            println!("Server started in {}ms", dur.as_millis());
        })?;

        Ok(TestServer { handle, addr })
    }

    async fn ensure_started(addr: SocketAddr) -> anyhow::Result<Duration> {
        // Wait for the server to be ready by attempting to connect
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(60);
        let mut sleep_time = Duration::from_millis(5);

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Server failed to start within timeout");
            }

            // Try to connect to see if the server is ready
            if TcpStream::connect(addr).await.is_ok() {
                return Ok(start.elapsed());
            }

            // Wait a bit before trying again
            tokio::time::sleep(sleep_time).await;

            sleep_time = sleep_time.mul_f64(1.5);
        }
    }

    /// Create a new client connection to the test server
    pub async fn connect(&self) -> anyhow::Result<TestClient> {
        let stream = TcpStream::connect(self.addr)
            .await
            .map_err(|e| anyhow!("Failed to connect to {}: {:?}", self.addr, e))?;
        Ok(TestClient { stream })
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Kill the server when test(s) completes
        self.handle.abort();
    }
}

pub struct TestClient {
    stream: TcpStream,
}

impl TestClient {
    /// Send a raw RESP command and read the response
    pub async fn send_command(&mut self, cmd: &[u8]) -> anyhow::Result<String> {
        self.stream.write_all(cmd).await?;
        self.stream.flush().await?;

        let mut buffer = vec![0u8; 4096];
        let n = self.stream.read(&mut buffer).await?;

        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }

    /// Helper to send RESP array commands
    pub async fn send_array(&mut self, args: &[&str]) -> anyhow::Result<String> {
        let cmd = encode_resp_array(args);
        self.send_command(cmd.as_bytes()).await
    }
}

/// Encode a RESP array command
pub fn encode_resp_array(args: &[&str]) -> String {
    let mut result = format!("*{}\r\n", args.len());
    for arg in args {
        result.push_str(&format!("${}\r\n{}\r\n", arg.len(), arg));
    }
    result
}

/// Parse a simple RESP string response
pub fn parse_simple_string(resp: &str) -> Option<&str> {
    if resp.starts_with('+') {
        Some(resp[1..].trim_end_matches("\r\n"))
    } else {
        None
    }
}

/// Parse a RESP integer response
pub fn parse_integer(resp: &str) -> Option<i64> {
    if resp.starts_with(':') {
        resp[1..].trim_end_matches("\r\n").parse().ok()
    } else {
        None
    }
}

/// Parse a RESP bulk string response
pub fn parse_bulk_string(resp: &str) -> Option<String> {
    if resp.starts_with('$') {
        let lines: Vec<&str> = resp.lines().collect();
        if lines.len() >= 2 {
            Some(lines[1].to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Parse a RESP array response
pub fn parse_array(resp: &str) -> Option<Vec<String>> {
    if resp.starts_with('*') {
        let lines: Vec<&str> = resp.lines().collect();
        if lines.is_empty() {
            return None;
        }

        let count: usize = lines[0][1..].parse().ok()?;
        let mut result = Vec::new();
        let mut i = 1;

        for _ in 0..count {
            if i < lines.len() && lines[i].starts_with('$') {
                i += 1; // Skip the bulk string length line
                if i < lines.len() {
                    result.push(lines[i].to_string());
                    i += 1;
                }
            }
        }

        Some(result)
    } else {
        None
    }
}
