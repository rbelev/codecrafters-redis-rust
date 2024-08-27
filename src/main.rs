mod resp;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::ops::Add;
use std::thread;
use std::str;
use std::time;
use std::time::{Duration, SystemTime};
use crate::resp::Value;

struct StoredValue {
    value: Value,
    expiry: Option<time::SystemTime>
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                thread::spawn(move || {
                    let mut read_buffer = [0;512];
                    let mut store: HashMap<String, StoredValue> = HashMap::new();

                    loop {
                        let read_count = stream.read(&mut read_buffer).unwrap();
                        if read_count == 0 {
                            break;
                        }
                        let command = str::from_utf8(&read_buffer).unwrap().to_string();

                        let response = process(command, &mut store);
                        stream.write(&*response.into_bytes()).unwrap();
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

/*
 * *1\r\n$4\r\nPING\r\n
 */
fn process(buff: String, store: &mut HashMap<String, StoredValue>) -> String {
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


fn eval_command(segments: &Value, store: &mut HashMap<String, StoredValue>) -> String {
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
        },
        _ => panic!("non-array command"),
    }
}

fn eval_set(params: &[Value], store: &mut HashMap<String, StoredValue>) -> String {
    println!("set params: {:?}", params);
    match params {
        [Value::BulkString(name), Value::BulkString(value)] => {
            store.insert(String::from(name), StoredValue { value: Value::BulkString(String::from(value)), expiry: None });
        },
        [
            Value::BulkString(name),
            Value::BulkString(value),
            Value::BulkString(_cmd),
            Value::BulkString(str_px)] => {
                let px = str_px.parse::<u64>().unwrap();
                store.insert(
                    String::from(name),
                 StoredValue {
                     value: Value::BulkString(String::from(value)),
                     expiry: Some(SystemTime::now().add(Duration::from_millis(px)))
                    }
                );
        },
        _ => {},
    };

    Value::SimpleString(String::from("OK")).serialize()
}
fn eval_get(params: &[Value], store: &HashMap<String, StoredValue>) -> String {
    println!("get params: {:?}", params);
    let value: Option<&StoredValue> = match params {
        [Value::BulkString(name)] => {
            store.get(name)
        },
        _ => None,
    };
    match value {
        Some(opt) => {
            match opt.expiry {
                Some(time) => {
                    if time::SystemTime::now() > time {
                        return String::from(Value::NULL_STRING);
                    }

                    opt.value.serialize()
                },
                None => opt.value.serialize(),
            }
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
