#![allow(dead_code)]
mod commands;
mod config;
mod db;
mod resp;

use crate::db::{DB, Redis};
use crate::resp::Value;
use std::error::Error;
use std::str;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn run_server(port: u16, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;

    let db = DB::new(args).parse_rdb();
    let redis: Redis = Arc::new(Mutex::new(db));

    loop {
        let (mut socket, _) = listener.accept().await?;
        let redis = redis.clone();

        tokio::spawn(async move {
            let mut read_buffer = [0; 512];

            loop {
                let read_count = match socket.read(&mut read_buffer).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {e:?}");
                        return;
                    }
                };
                let Ok(command) = str::from_utf8(&read_buffer[..read_count]) else {
                    eprintln!("not utf8 buffer");
                    return;
                };

                let response = match process(command, &redis).await {
                    Ok(response) => response,
                    Err(err) => {
                        eprintln!("failed to process command {err}");
                        return;
                    }
                };
                if let Err(e) = socket.write_all(&response.into_bytes()).await {
                    eprintln!("failed to write to socket; err = {e}");
                    return;
                };
            }
        });
    }
}

/*
 * *1\r\n$4\r\nPING\r\n
 */
async fn process(buff: &str, store: &Redis) -> Result<String, String> {
    let mut lines = buff.lines();
    let Some(parsed_value) = Value::parse(&mut lines) else {
        return Err("No value parsed".to_string());
    };

    let store = store.lock().unwrap();
    let Ok(response) = commands::eval_command(&parsed_value, store) else {
        return Ok("-ERR".to_string());
    };
    Ok(response.serialize())
}
