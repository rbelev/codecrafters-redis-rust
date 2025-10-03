use crate::db::DB;
use crate::resp::Value;
use std::sync::MutexGuard;

pub fn eval_echo(params: &[Value]) -> Result<Value, String> {
    Ok(params[0].clone())
}

pub fn eval_ping(params: &[Value]) -> Result<Value, String> {
    let Some(message) = params.first() else {
        return Ok(Value::SimpleString("PONG".to_string()));
    };
    Ok(message.clone())
}

pub fn eval_config(params: &[Value], store: MutexGuard<DB>) -> Result<Value, String> {
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

pub fn eval_keys(params: &[Value], store: MutexGuard<DB>) -> Result<Value, String> {
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
