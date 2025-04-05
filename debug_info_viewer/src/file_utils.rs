use std::path::Path;
use std::fs;


pub fn read_file(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(
        |err| {
            eprintln!("Could not read file `{}`\n{}", path.to_string_lossy(), err);
            std::process::exit(1);
        }
    )
}
