use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct Config {
    pub dir: String,
    pub dbfilename: String,
}

impl Config {
    pub const DEFAULT_DATA_DIR: &'static str = "/tmp/redis-data";
    pub const DEFAULT_DATA_FILE: &'static str = "rdbfile.rdb";

    pub fn new(args: Vec<String>) -> Config {
        let mut parsed: HashMap<String, String> = HashMap::new();
        let mut iter = args.into_iter();

        while let Some(flag) = iter.next() {
            if flag.starts_with("--") {
                if let Some(value) = iter.next() {
                    parsed.insert(flag, value);
                }
            }
        }

        Config {
            dir: parsed
                .remove("--dir")
                .unwrap_or_else(|| String::from(Self::DEFAULT_DATA_DIR)),
            dbfilename: parsed
                .remove("--dbfilename")
                .unwrap_or_else(|| String::from(Self::DEFAULT_DATA_FILE)),
            // ..Self::default()
        }
    }

    pub fn rbd(&self) -> PathBuf {
        Path::new(&self.dir).join(&self.dbfilename)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dir: String::from(Self::DEFAULT_DATA_DIR),
            dbfilename: String::from(Self::DEFAULT_DATA_FILE),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_args() {
        let config = Config::new(Vec::new());
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_custom_args() {
        let my_dir = String::from("/tmp/my-data");
        let my_db_file = String::from("my_redis.rdb");
        let args = vec![
            String::from("--dir"),
            my_dir.clone(),
            String::from("--dbfilename"),
            my_db_file.clone(),
        ];

        let config = Config::new(args);
        assert_eq!(
            config,
            Config {
                dir: my_dir,
                dbfilename: my_db_file,
            }
        );
    }

    #[test]
    fn test_partial_args() {
        let my_dir = String::from("/tmp/my-data-2");
        let config = Config::new(vec!["--dir".to_string(), my_dir.clone()]);

        assert_eq!(
            config,
            Config {
                dir: my_dir,
                ..Default::default()
            }
        )
    }
}
