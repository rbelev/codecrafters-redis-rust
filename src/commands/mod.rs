mod basics;
mod lists;
mod numbers;
mod strings;

use crate::db::DB;
use crate::resp::Value;
use std::sync::MutexGuard;

pub fn eval_command(segments: &Value, store: MutexGuard<DB>) -> Result<Value, String> {
    match segments {
        Value::Array(arr) => {
            match &arr[0] {
                Value::BulkString(cmd) if cmd == "ECHO" => basics::eval_echo(&arr[1..]),
                Value::BulkString(cmd) if cmd == "PING" => basics::eval_ping(&arr[1..]),
                Value::BulkString(cmd) if cmd == "CONFIG" => basics::eval_config(&arr[1..], store),
                Value::BulkString(cmd) if cmd == "KEYS" => basics::eval_keys(&arr[1..], store),

                Value::BulkString(cmd) if cmd == "SET" => strings::eval_set(&arr[1..], store),
                Value::BulkString(cmd) if cmd == "GET" => strings::eval_get(&arr[1..], store),

                Value::BulkString(cmd) if cmd == "RPUSH" => lists::rpush(&arr[1..], store),

                // Value::BulkString(cmd) if cmd == "INCR" => eval_incr(&arr[1..], store),
                Value::BulkString(cmd) => Err(format!("Not a valid command: {cmd}")),
                _ => Err(format!("non-BulkString first: {}", &arr[0].serialize())),
            }
        }
        _ => Err("non-array command".to_string()),
    }
}
