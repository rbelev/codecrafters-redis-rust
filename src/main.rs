mod resp;

use crate::resp::Value;
use std::collections::HashMap;
use std::ops::Add;
use std::str;
use std::sync::{Arc, Mutex};
use std::time;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

type Db = Arc<Mutex<HashMap<String, StoredValue>>>;

struct StoredValue {
    value: Value,
    expiry: Option<time::SystemTime>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (mut socket, _) = listener.accept().await?;
        let db = db.clone();

        tokio::spawn(async move {
            let mut read_buffer = [0; 512];

            loop {
                let read_count = match socket.read(&mut read_buffer).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                let command = str::from_utf8(&read_buffer[..read_count]).unwrap();

                let db = db.clone();

                let response = process(command, db);
                if let Err(e) = socket.write_all(&response.into_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                };
            }
        });
    }
}

/*
 * *1\r\n$4\r\nPING\r\n
 */
fn process(buff: &str, store: Db) -> String {
    println!("process: {}", buff);

    let mut lines = buff.split("\r\n");
    let parsed_value = Value::parse(&mut lines);
    println!("parsed: {:?}", parsed_value);
    eval_command(&parsed_value, store)

    // match parsed_value {
    //     Value::Array => {
    //         eval_command(&parsed_value)
    //     },
    //     _ => panic!("commands from client always expected to be an array")
    // }
}

fn eval_command(segments: &Value, store: Db) -> String {
    println!("eval_command: {:?}", segments);
    match segments {
        Value::Array(arr) => {
            println!("eval_command array: {:?}", arr);

            match &arr[0] {
                Value::BulkString(cmd) if cmd == "ECHO" => eval_echo(&arr[1..]),
                Value::BulkString(cmd) if cmd == "PING" => eval_ping(),
                Value::BulkString(cmd) if cmd == "SET" => eval_set(&arr[1..], store),
                Value::BulkString(cmd) if cmd == "GET" => eval_get(&arr[1..], store),
                Value::BulkString(cmd) => panic!("Not a valid command: {}", cmd),
                _ => panic!("non-simple string first: {}", &arr[0].serialize()),
            }
        }
        _ => panic!("non-array command"),
    }
}

fn eval_set(params: &[Value], store: Db) -> String {
    println!("set params: {:?}", params);
    let mut store = store.lock().unwrap();

    match params {
        [Value::BulkString(name), Value::BulkString(value)] => {
            store.insert(
                String::from(name),
                StoredValue {
                    value: Value::BulkString(String::from(value)),
                    expiry: None,
                },
            );
        }
        [Value::BulkString(name), Value::BulkString(value), Value::BulkString(_cmd), Value::BulkString(str_px)] =>
        {
            let px = str_px.parse::<u64>().unwrap();
            store.insert(
                String::from(name),
                StoredValue {
                    value: Value::BulkString(String::from(value)),
                    expiry: Some(SystemTime::now().add(Duration::from_millis(px))),
                },
            );
        }
        _ => {}
    };

    Value::SimpleString(String::from("OK")).serialize()
}
fn eval_get(params: &[Value], store: Db) -> String {
    println!("get params: {:?}", params);
    let store = store.lock().unwrap();

    let value: Option<&StoredValue> = match params {
        [Value::BulkString(name)] => store.get(name),
        _ => None,
    };
    match value {
        Some(opt) => match opt.expiry {
            Some(time) => {
                if time::SystemTime::now() > time {
                    return String::from(Value::NULL_STRING);
                }

                opt.value.serialize()
            }
            None => opt.value.serialize(),
        },
        None => String::from(Value::NULL_STRING),
    }

    // Value::NULL_STRING

    // Value::BulkString(String::from(value)).serialize()
}

fn eval_echo(params: &[Value]) -> String {
    params[0].serialize()
}

fn eval_ping() -> String {
    Value::SimpleString(String::from("PONG")).serialize()
}
