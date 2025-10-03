use crate::db::{DB, StoredValue};
use crate::resp::Value;
use std::sync::MutexGuard;

pub fn rpush(params: &[Value], mut store: MutexGuard<DB>) -> Result<Value, String> {
    if params.len() < 2 {
        return Err("RPUSH requires at least 2 arguments".to_string());
    }

    let Value::BulkString(list_name) = &params[0] else {
        return Err("Bad args given to RPUSH".to_string());
    };

    // Get or create the list in one operation
    let entry = store
        .db
        .entry(list_name.to_string())
        .or_insert(StoredValue {
            value: Value::Array(vec![]),
            expiry: None,
        });
    let Value::Array(list) = &mut entry.value else {
        return Err(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        );
    };

    // Push all elements
    for elem in &params[1..] {
        list.push(elem.clone());
    }
    Ok(Value::Integer(list.len() as i64))
}
