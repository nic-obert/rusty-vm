use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;


lazy_static! {

    pub static ref INCLUDE_LIB_PATH: PathBuf = {
        match env::var("EASYVM_INCLUDE_LIB") {
            Ok(path) => PathBuf::from_str(&path).unwrap(),
            Err(_) => PathBuf::from_str("assembler/lib").unwrap(),
        }
    };

}

