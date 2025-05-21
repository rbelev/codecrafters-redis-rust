use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Config {
    pub dir: String,
    pub dbfilename: String,
}

impl Config {
    pub fn new(mut args: Vec<String>) -> Config {
        // println!("{args:?}");
        let dir: String = match args.iter().position(|arg| arg == "--dir") {
            Some(pos) => args.remove(pos + 1),
            None => String::from("/tmp/redis-data"),
        };
        let dbfilename: String = match args.iter().position(|arg| arg == "--dbfilename") {
            Some(pos) => args.remove(pos + 1),
            None => String::from("rdbfile.rdb"),
        };
        Config { dir, dbfilename }
    }

    pub fn rbd(&self) -> PathBuf {
        Path::new(&self.dir).join(&self.dbfilename)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl PartialEq for Config {
        fn eq(&self, other: &Self) -> bool {
            self.dir == other.dir && self.dbfilename == other.dbfilename
        }
    }
    #[test]
    fn test_args() {
        let args = vec![
            String::from("--dir"),
            String::from("/tmp/my-data"),
            String::from("--dbfilename"),
            String::from("my_redis.rdb"),
        ];
        let config = Config::new(args);
        assert_eq!(
            config,
            Config {
                dir: String::from("/tmp/my-data"),
                dbfilename: String::from("my_redis.rdb"),
            }
        );
    }
}
