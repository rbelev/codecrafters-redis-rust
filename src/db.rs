use crate::resp::Value;
use std::collections::HashMap;
use std::env::Args;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub type Redis = Arc<Mutex<DB>>;

pub struct DB {
    pub config: Config,
    pub db: HashMap<String, StoredValue>,
}

pub struct Config {
    pub dir: String,
    pub dbfilename: String,
}

impl Config {
    pub fn new(args: &mut Args) -> Config {
        let mut build_config: HashMap<String, String> = HashMap::new();

        loop {
            match args.next() {
                Some(arg) if arg == "--dir" || arg == "--dbfilename" => {
                    build_config.insert(arg, args.next().unwrap().clone());
                }
                Some(arg) => panic!("unknown arg: {}", arg),
                None => break,
            };
        }

        let dir: String = match build_config.get("--dir") {
            Some(dir) => dir.clone(),
            None => String::from("/tmp/redis-data"),
        };
        let dbfilename: String = match build_config.get("--dbfilename") {
            Some(file) => file.clone(),
            None => String::from("rdbfile.rdb"),
        };

        Config { dir, dbfilename }
    }
}

pub struct StoredValue {
    pub value: Value,
    pub expiry: Option<SystemTime>,
}

impl DB {
    pub fn new(args: &mut Args) -> DB {
        DB {
            config: Config::new(args),
            db: HashMap::new(),
        }
    }
}
