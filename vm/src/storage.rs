use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::path::PathBuf;
use std::io;

use rust_vm_lib::vm::ErrorCodes;

use crate::error;


pub struct Storage {

    file_path: PathBuf,
    file: File,
    max_size: Option<usize>,

}


impl Storage {

    pub fn new(file_path: PathBuf, max_size: Option<usize>) -> Self {

        let file: File = if file_path.exists() {
            let file = File::open(&file_path).unwrap_or_else(
                |err| error::io_error(&file_path, &err, format!("Failed to open storage file \"{}\"", file_path.display()).as_str())
            );
            
            if file.metadata().unwrap_or_else(
                |err| error::io_error(&file_path, &err, format!("Failed to get metadata of storage file \"{}\"", file_path.display()).as_str())
            ).len() > max_size.unwrap_or(usize::MAX) as u64 {
                error::error(format!("Storage file \"{}\" is too big", file_path.display()).as_str());
            }

            file

        } else {
            File::create(&file_path).unwrap_or_else(
                |err| error::io_error(&file_path, &err, format!("Failed to create storage file \"{}\"", file_path.display()).as_str())
            )
        };
        
        Self {
            file_path: file_path.canonicalize().unwrap_or_else(
                |err| error::io_error(&file_path, &err, format!("Failed to canonicalize path \"{}\"", file_path.display()).as_str())
            ),
            file,
            max_size,
        }
    }


    /// Try to read `size` bytes from the storage file at `offset`.
    pub fn read(&self, offset: usize, size: usize) -> Result<Vec<u8>, ErrorCodes> {

        let mut buffer = vec![0; size];

        match self.file.read_exact_at(&mut buffer, offset as u64) {

            Ok(_) => Ok(buffer),
            
            Err(err) => Err(match err.kind() {
                io::ErrorKind::UnexpectedEof => ErrorCodes::EndOfFile,
                _ => error::io_error(&self.file_path, &err, format!("Failed to read storage file \"{}\"", self.file_path.display()).as_str())
            })
        }
    }


    /// Try to write `data` to the storage file at `offset`.
    pub fn write(&self, offset: usize, data: &[u8]) -> Result<(), ErrorCodes> {

        match self.file.write_all_at(data, offset as u64) {

            Ok(_) => {
                if let Some(max_size) = self.max_size {
                    if self.file.metadata().unwrap_or_else(
                        |err| error::io_error(&self.file_path, &err, format!("Failed to get metadata of storage file \"{}\"", self.file_path.display()).as_str())
                    ).len() > max_size as u64 {
                        return Err(ErrorCodes::OutOfBounds);
                    }
                }
                Ok(())
            },
            
            Err(err) => error::io_error(&self.file_path, &err, format!("Failed to write storage file \"{}\"", self.file_path.display()).as_str())

        }
    }


}

