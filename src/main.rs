mod commands;
mod config;
mod db;
mod resp;

use crate::db::{Redis, DB};
use crate::resp::Value;
use std::env;
use std::error::Error;
use std::ops::Add;
use std::str;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let args = env::args().skip(1).collect::<Vec<String>>();
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
    let parsed_value = Value::parse(&mut lines);

    let store = store.lock().unwrap();
    let response = eval_command(&parsed_value, store);
    Ok(response?.serialize())
}

fn eval_command(segments: &Value, store: MutexGuard<DB>) -> Result<Value, String> {
    match segments {
        Value::Array(arr) => {
            match &arr[0] {
                Value::BulkString(cmd) if cmd == "ECHO" => eval_echo(&arr[1..]),
                Value::BulkString(cmd) if cmd == "PING" => eval_ping(),
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
        [Value::BulkString(name), Value::BulkString(value), Value::BulkString(_cmd), Value::BulkString(str_px)] =>
        {
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
        _ => {}
    };

    Ok(Value::SimpleString("OK".to_string()))
}
fn eval_get(params: &[Value], store: MutexGuard<DB>) -> Result<Value, String> {
    let value: Option<&db::StoredValue> = match params.first() {
        Some(Value::BulkString(name)) => store.db.get(name),
        _ => None,
    };

    match value {
        Some(opt) if !opt.is_expired() => Ok(opt.value.clone()),
        _ => Ok(Value::SimpleString(Value::NULL_STRING.to_string())),
    }

    /* let chain for rust 1.88 2024 version.
    if let Some(Value::BulkString(val)) = params.first()
        && let Some(stored) = store.db.get(val)
        && let Some(value) = stored.get()
    {
        value.clone()
    } else {
        Value::SimpleString(Value::NULL_STRING.to_string())
    }
    */
}

fn eval_echo(params: &[Value]) -> Result<Value, String> {
    Ok(params[0].clone())
}

fn eval_ping() -> Result<Value, String> {
    Ok(Value::SimpleString("PONG".to_string()))
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
