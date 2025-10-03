#![allow(dead_code)]
mod commands;
mod config;
mod db;
mod resp;

use crate::db::{DB, Redis};
use crate::resp::Value;
use std::error::Error;
use std::ops::Add;
use std::str;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime};
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
                let command = str::from_utf8(&read_buffer[..read_count]).expect("utf8 buffer");

                let response = process(command, &redis).unwrap();
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
fn process(buff: &str, store: &Redis) -> Result<String, String> {
    let mut lines = buff.lines();
    let Some(parsed_value) = Value::parse(&mut lines) else {
        return Err("No value parsed".to_string());
    };

    let store = store.lock().unwrap();
    let Ok(response) = eval_command(&parsed_value, store) else {
        return Ok("-ERR".to_string());
    };
    Ok(response.serialize())
}

fn eval_command(segments: &Value, store: MutexGuard<DB>) -> Result<Value, String> {
    match segments {
        Value::Array(arr) => {
            match &arr[0] {
                Value::BulkString(cmd) if cmd == "ECHO" => eval_echo(&arr[1..]),
                Value::BulkString(cmd) if cmd == "PING" => eval_ping(&arr[1..]),
                Value::BulkString(cmd) if cmd == "SET" => eval_set(&arr[1..], store),
                Value::BulkString(cmd) if cmd == "GET" => eval_get(&arr[1..], store),
                Value::BulkString(cmd) if cmd == "KEYS" => eval_keys(&arr[1..], store),
                Value::BulkString(cmd) if cmd == "CONFIG" => eval_config(&arr[1..], store),
                // Value::BulkString(cmd) if cmd == "INCR" => eval_incr(&arr[1..], store),
                Value::BulkString(cmd) if cmd == "RPUSH" => commands::rpush(&arr[1..], store),
                Value::BulkString(cmd) => Err(format!("Not a valid command: {cmd}")),
                _ => Err(format!("non-BulkString first: {}", &arr[0].serialize())),
            }
        }
        _ => Err("non-array command".to_string()),
    }
}

fn eval_set(params: &[Value], mut store: MutexGuard<DB>) -> Result<Value, String> {
    println!("set params: {params:?}");

    match params {
        [Value::BulkString(name), Value::BulkString(value)] => {
            store.db.insert(
                String::from(name),
                db::StoredValue {
                    value: Value::BulkString(String::from(value)),
                    expiry: None,
                },
            );
        }
        [
            Value::BulkString(name),
            Value::BulkString(value),
            Value::BulkString(_cmd),
            Value::BulkString(str_px),
        ] => {
            let px = str_px
                .parse::<u64>()
                .map_err(|err| format!("invalid px: {err}"))?;
            store.db.insert(
                String::from(name),
                db::StoredValue {
                    value: Value::BulkString(String::from(value)),
                    expiry: Some(SystemTime::now().add(Duration::from_millis(px))),
                },
            );
        }
        _ => {
            return Err("invalid number of arguments".to_string());
        }
    };

    Ok(Value::SimpleString("OK".to_string()))
}
fn eval_get(params: &[Value], store: MutexGuard<DB>) -> Result<Value, String> {
    if let Some(Value::BulkString(val)) = params.first()
        && let Some(stored) = store.db.get(val)
        && let Some(value) = stored.get()
    {
        Ok(value.clone())
    } else {
        Ok(Value::NullString)
    }
}

fn eval_echo(params: &[Value]) -> Result<Value, String> {
    Ok(params[0].clone())
}

fn eval_ping(params: &[Value]) -> Result<Value, String> {
    let Some(message) = params.first() else {
        return Ok(Value::SimpleString("PONG".to_string()));
    };
    Ok(message.clone())
}

fn eval_config(params: &[Value], store: MutexGuard<DB>) -> Result<Value, String> {
    // Assumed GET, so skipping past [0].
    let field = params[1].clone();
    let config_value = match &field {
        Value::BulkString(tar) if tar == "dir" => &store.config.dir,
        Value::BulkString(tar) if tar == "dbfilename" => &store.config.dbfilename,
        bad_tar => {
            return Err(format!("unknown config: {}", bad_tar.serialize()));
        }
    };

    Ok(Value::Array(vec![
        field,
        Value::BulkString(config_value.clone()),
    ]))
}

fn eval_keys(params: &[Value], store: MutexGuard<DB>) -> Result<Value, String> {
    match &params[0] {
        Value::BulkString(all) if all == "*" => {
            let all = store
                .db
                .keys()
                .map(|key| Value::BulkString(key.clone()))
                .collect::<Vec<Value>>();
            Ok(Value::Array(all))
        }
        _ => panic!("eval_keys: only * is supported: {params:?}"),
    }
}

// fn eval_incr(params: &[Value], store: &Redis) -> String {
//     let store = store.lock().unwrap();
//     let key = params.get(0);
//
//     let value: Option<&db::StoredValue> = match key {
//         Some(Value::BulkString(name)) => store.db.get(name),
//         _ => None,
//     };
//     match value {
//         None => {
//             store.db.insert(params.get(0), 1);
//         }
//         Some(Value::Integer(integer)) => {
//             store.db.insert(params.get(0), integer + 1);
//         }
//     }
// }
