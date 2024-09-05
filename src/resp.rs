#[derive(Debug, Clone)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Integer(i64),
    Array(Vec<Value>),
}

impl Value {
    pub const NULL_STRING: &'static str = "$-1\r\n";

    pub fn serialize(self: &Self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            Value::Integer(i) => format!(":{}\r\n", i),
            Value::Array(v) => {
                let serialize_values = v
                    .iter()
                    .map(|value: &Value| value.serialize())
                    .collect::<Vec<String>>()
                    .join("");

                format!("*{}\r\n{}", v.len(), serialize_values)
            }
        }
    }

    /*
     * *1\r\n$4\r\nPING\r\n
     */
    pub fn parse(iter: &mut dyn Iterator<Item = &str>) -> Value {
        let line = iter.next().unwrap();
        let symbol = line.chars().next().unwrap();

        match symbol {
            '+' => Self::parse_simple_string(iter, line),
            '*' => Self::parse_array(iter, line),
            '$' => Self::parse_bulk_string(iter, line),
            ':' => Self::parse_integer(iter, line),
            _ => panic!("unsupported command"),
        }
    }

    fn parse_simple_string(_iter: &mut dyn Iterator<Item = &str>, line: &str) -> Value {
        Value::SimpleString(line[1..].to_string())
    }

    fn parse_bulk_string(iter: &mut dyn Iterator<Item = &str>, line: &str) -> Value {
        let _bytes: u64 = line[1..].to_string().parse().unwrap();
        let word = iter.next().unwrap().to_string();
        Value::BulkString(word)
    }
    fn parse_integer(_iter: &mut dyn Iterator<Item = &str>, line: &str) -> Value {
        Value::Integer(line[1..].to_string().parse().unwrap())
    }

    fn parse_array(iter: &mut dyn Iterator<Item = &str>, line: &str) -> Value {
        let array_length = line[1..].to_string().parse().unwrap();
        let mut arr: Vec<Value> = Vec::with_capacity(array_length);

        for _i in 1..=array_length {
            let next_value = Self::parse(iter);
            arr.push(next_value);
        }
        Value::Array(arr)
    }
}
