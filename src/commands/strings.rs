use crate::db;
use crate::db::DB;
use crate::resp::Value;
use std::ops::Add;
use std::sync::MutexGuard;
use std::time::{Duration, SystemTime};

pub fn eval_set(params: &[Value], mut store: MutexGuard<DB>) -> Result<Value, String> {
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

pub fn eval_get(params: &[Value], store: MutexGuard<DB>) -> Result<Value, String> {
    if let Some(Value::BulkString(val)) = params.first()
        && let Some(stored) = store.db.get(val)
        && let Some(value) = stored.get()
    {
        Ok(value.clone())
    } else {
        Ok(Value::NullString)
    }
}
