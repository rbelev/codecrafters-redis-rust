use crate::db;
use crate::db::Redis;
use crate::resp::Value;

pub fn eval_incr(params: &[Value], store: &Redis) -> String {
    let store = store.lock().unwrap();
    let key = params.get(0);

    let _value: Option<&db::StoredValue> = match key {
        Some(Value::BulkString(name)) => store.db.get(name),
        _ => None,
    };
    unimplemented!()
    // match value {
    //     None => {
    //         store.db.insert(params.get(0), 1);
    //     }
    //     Some(Value::Integer(integer)) => {
    //         store.db.insert(params.get(0), integer + 1);
    //     }
    // }
}
