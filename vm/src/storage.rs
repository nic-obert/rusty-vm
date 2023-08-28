use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::path::PathBuf;


pub struct Storage {

    file_path: PathBuf,
    file: File,
    max_size: Option<usize>,

}


impl Storage {

    pub fn new(file_path: PathBuf, max_size: Option<usize>) -> Self {

        let file: File = if file_path.exists() {
            let file = File::open(&file_path).unwrap_or_else(
                |err| crate::error::io_error(&file_path, &err, format!("Failed to open storage file \"{}\"", file_path.display()).as_str())
            );
            
            if file.metadata().unwrap_or_else(
                |err| crate::error::io_error(&file_path, &err, format!("Failed to get metadata of storage file \"{}\"", file_path.display()).as_str())
            ).len() > max_size.unwrap_or(usize::MAX) as u64 {
                crate::error::error(format!("Storage file \"{}\" is too big", file_path.display()).as_str());
            }

            file

        } else {
            File::create(&file_path).unwrap_or_else(
                |err| crate::error::io_error(&file_path, &err, format!("Failed to create storage file \"{}\"", file_path.display()).as_str())
            )
        };
        
        Self {
            file_path: file_path.canonicalize().unwrap_or_else(
                |err| crate::error::io_error(&file_path, &err, format!("Failed to canonicalize path \"{}\"", file_path.display()).as_str())
            ),
            file,
            max_size,
        }
    }


    /// Try to read `size` bytes from the storage file at `offset`.
    pub fn read(&self, offset: usize, size: usize) -> Option<Vec<u8>> {

        let mut buffer = vec![0; size];

        // TODO: check out of bounds

        if let Err(err) = self.file.read_exact_at(&mut buffer, offset as u64) {
            crate::error::io_error(&self.file_path, &err, format!("Failed to read storage file \"{}\"", self.file_path.display()).as_str());
        }

        Some(buffer)
    }


    /// Try to write `data` to the storage file at `offset`.
    pub fn write(&mut self, offset: usize, data: &[u8]) {

        if let Err(err) = self.file.write_all_at(data, offset as u64) {
            crate::error::io_error(&self.file_path, &err, format!("Failed to write storage file \"{}\"", self.file_path.display()).as_str());
        }

        // TODO: Check if the file is too big
    }


}

