#[derive(Debug, Clone)]
pub enum Value {
    NullString,
    SimpleString(String),
    BulkString(String),
    Integer(i64),
    Array(Vec<Value>),
}

impl Value {
    pub const NULL_STRING: &'static str = "$-1\r\n";

    pub fn serialize(&self) -> String {
        match self {
            Value::NullString => Value::NULL_STRING.to_string(),
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            Value::Integer(i) => format!(":{}\r\n", i),
            Value::Array(v) => {
                let serialize_values = v
                    .iter()
                    .map(|value: &Value| value.serialize())
                    .collect::<String>();

                format!("*{}\r\n{}", v.len(), serialize_values)
            }
        }
    }

    /*
     * *1\r\n$4\r\nPING\r\n
     */
    pub fn parse<'a, I>(iter: &mut I) -> Option<Value>
    where
        I: Iterator<Item = &'a str>,
    {
        let line = iter.next()?;

        match line.chars().next() {
            Some('+') => Self::parse_simple_string(iter, line),
            Some('*') => Self::parse_array(iter, line),
            Some('$') => Self::parse_bulk_string(iter, line),
            Some(':') => Self::parse_integer(iter, line),
            _ => None,
        }
    }

    fn parse_simple_string<'a, I>(_iter: &mut I, line: &str) -> Option<Value>
    where
        I: Iterator<Item = &'a str>,
    {
        Some(Value::SimpleString(line[1..].to_string()))
    }

    fn parse_bulk_string<'a, I>(iter: &mut I, line: &str) -> Option<Value>
    where
        I: Iterator<Item = &'a str>,
    {
        let Ok(_bytes) = line[1..].parse::<u64>() else {
            return None;
        };
        let word = iter.next()?.to_string();
        Some(Value::BulkString(word))
    }

    fn parse_integer<'a, I>(_iter: I, line: &str) -> Option<Value>
    where
        I: Iterator<Item = &'a str>,
    {
        let Ok(integer) = line[1..].parse::<i64>() else {
            return None;
        };
        Some(Value::Integer(integer))
    }

    fn parse_array<'a, I>(iter: &mut I, line: &str) -> Option<Value>
    where
        I: Iterator<Item = &'a str>,
    {
        let Ok(array_length) = line[1..].parse::<usize>() else {
            return None;
        };
        let mut arr: Vec<Value> = Vec::with_capacity(array_length);

        for _i in 1..=array_length {
            let next_value = Self::parse(iter)?;
            arr.push(next_value);
        }
        Some(Value::Array(arr))
    }
}
