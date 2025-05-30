use crate::config::Config;
use crate::resp::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::iter::Peekable;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type Redis = Arc<Mutex<DB>>;

pub struct DB {
    pub config: Config,
    pub db: HashMap<String, StoredValue>,
}

pub struct StoredValue {
    pub value: Value,
    pub expiry: Option<SystemTime>,
}

impl DB {
    pub fn new(args: Vec<String>) -> Self {
        DB {
            config: Config::new(args),
            db: HashMap::new(),
        }
    }

    fn load_rdb(&self) -> Vec<u8> {
        let path = self.config.rbd();
        fs::read(path).unwrap_or_else(|_err| {
            vec![
                0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31, // Header
                0xFA, // Meta 1
                0x09, 0x72, 0x65, 0x64, 0x69, 0x73, 0x2D, 0x76, 0x65, 0x72, // Meta 1 Key
                0x05, 0x37, 0x2E, 0x32, 0x2E, 0x30, // Meta 1 Value
                0xFA, // Meta 2
                0x0A, 0x72, 0x65, 0x64, 0x69, 0x73, 0x2D, 0x62, 0x69, 0x74,
                0x73, // Meta 2 Key
                0xC0, 0x40, // Meta 2 Value
                // C = 1100 -- special format string encoding, type = 00_3F
                0xFE, 0x00, // Database Index 00
                0xFB, 0x01, 0x00, // Resize_db field
                0x00, 0x06, 0x62, 0x61, 0x6E, 0x61, 0x6E, 0x61, 0x05, 0x6D, 0x61, 0x6E, 0x67, 0x6F,
                0xFF, // End of RDB file indicator
                // 8 byte Checksum?? But have 9 leftover.
                0x53, 0x19, 0x39, 0x63, 0x07, 0xDB, 0x0D, 0xC0, 0x0A,
            ]
        })
    }

    pub fn parse_rdb(mut self) -> Self {
        let rdb = self.load_rdb();
        // println!("{rdb:#04X?}");

        let mut byte_cursor: Peekable<_> = rdb
            .iter()
            .copied()
            .skip_while(|&x| x != 0xFE)
            .skip(1)
            .peekable();

        let database_index = byte_cursor.next().unwrap();
        println!("starting database #{}", database_index);

        if byte_cursor.next() != Some(0xFB) {
            panic!("expected FB after db start");
        }
        let _hash_table_size = Self::get_length_encoding(&mut byte_cursor);
        let _expire_table_size = Self::get_length_encoding(&mut byte_cursor);

        loop {
            if let Some(0xFF) = byte_cursor.peek() {
                byte_cursor.next();
                break;
            }

            let expiry = Self::get_expiry(&mut byte_cursor);
            let value_type = byte_cursor.next().unwrap();
            let key = Self::get_string(&mut byte_cursor);
            let value = Self::get_string(&mut byte_cursor);

            println!("inserting {key}:{value} {expiry:?} type={value_type:#04X?}");

            self.db.insert(
                key,
                StoredValue {
                    value: Value::SimpleString(value),
                    expiry,
                },
            );
        }

        self
    }

    fn get_length_encoding(iter: &mut dyn Iterator<Item = u8>) -> u64 {
        let n = iter.next().unwrap();
        match n & 0xC0 {
            0x00 => (n & 0x3F) as u64,
            0x40 => {
                let n2 = iter.next().unwrap() as u64;

                ((n & 0x3F) as u64) << 8 | n2
            }
            0x80 => Self::extract_u32(iter) as u64,
            0xC0 => {
                // Special encoding:
                match n & 0x03 {
                    0x00 => iter.next().unwrap() as u64,
                    0x01 => Self::extract_u16(iter) as u64,
                    0x10 => Self::extract_u32(iter) as u64,
                    _ => panic!(),
                }
            }
            _ => panic!("Unexpected length encoding"),
        }
    }

    fn get_expiry<I: Iterator<Item = u8>>(iter: &mut Peekable<I>) -> Option<SystemTime> {
        match iter.peek() {
            Some(0xFD) => {
                iter.next();
                // FD $unsigned-int            # "expiry time in seconds", followed by 4 byte unsigned int
                let expiry_sec = Self::extract_u32(iter);
                println!("Expiry sec: {expiry_sec}");
                Some(UNIX_EPOCH + Duration::from_secs(expiry_sec as u64))
            }
            Some(0xFC) => {
                iter.next();
                // FC $unsigned long           # "expiry time in ms", followed by 8 byte unsigned long
                let expiry_ms = Self::extract_u64(iter);
                println!("Expiry ms: {expiry_ms}");
                Some(UNIX_EPOCH + Duration::from_millis(expiry_ms))
            }
            _ => None,
        }
    }

    fn get_string(iter: &mut dyn Iterator<Item = u8>) -> String {
        let size = Self::get_length_encoding(iter);
        // println!("get_string: attempting {size:?} bytes");
        let mut res = String::with_capacity(size as usize);
        for _ in 0..size {
            res.push(iter.next().unwrap() as char);
        }
        res
    }

    fn extract_u16(iter: &mut dyn Iterator<Item = u8>) -> u16 {
        let bytes = [iter.next().unwrap(), iter.next().unwrap()];
        u16::from_le_bytes(bytes)
    }
    fn extract_u32(iter: &mut dyn Iterator<Item = u8>) -> u32 {
        let bytes = [
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ];
        u32::from_le_bytes(bytes)
    }
    fn extract_u64(iter: &mut dyn Iterator<Item = u8>) -> u64 {
        let bytes = [
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ];
        u64::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_rdb_test() {
        let db = DB::new(vec![]);
        db.parse_rdb();
    }
}
