use std::fs::{File, OpenOptions, self};
use std::os::unix::prelude::FileExt;
use std::path::{PathBuf, Path};
use std::io;

use rusty_vm_lib::vm::ErrorCodes;

use crate::error;


pub struct Storage {

    file_path: PathBuf,
    file: File,
    max_size: Option<usize>,

}


impl Storage {

    pub fn new(file_path: PathBuf, max_size: Option<usize>) -> Self {

        let file: File = if file_path.exists() {
            let file = OpenOptions::new()
                .create(false)
                .write(true)
                .read(true)
                .open(&file_path)
                .unwrap_or_else(
                |err| error::io_error(&file_path, &err, format!("Failed to open storage file \"{}\"", file_path.display()).as_str())
            );
            
            if file.metadata().unwrap_or_else(
                |err| error::io_error(&file_path, &err, format!("Failed to get metadata of storage file \"{}\"", file_path.display()).as_str())
            ).len() > max_size.unwrap_or(usize::MAX) as u64 {
                error::error(format!("Storage file \"{}\" is too big", file_path.display()).as_str());
            }

            file

        } else {
            fs::create_dir_all(Path::new(&file_path).parent().unwrap_or_else(
                || error::error(format!("Failed to get parent directory of storage file \"{}\"", file_path.display()).as_str())
            )).unwrap_or_else(
                |err| error::io_error(&file_path, &err, format!("Failed to create parent directory of storage file \"{}\"", file_path.display()).as_str())
            );

            OpenOptions::new()  

                .create_new(true)
                .write(true)
                .read(true)
                .open(&file_path)
                .unwrap_or_else(
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

        if let Some(max_size) = self.max_size {
            if offset + data.len() > max_size {
                return Err(ErrorCodes::OutOfBounds);
            }
        }

        match self.file.write_all_at(data, offset as u64) {

            Ok(_) => {
                self.file.sync_all().unwrap_or_else(
                    |err| error::io_error(&self.file_path, &err, format!("Failed to sync storage file \"{}\"", self.file_path.display()).as_str())
                );

                Ok(())
            },
            
            Err(err) => error::io_error(&self.file_path, &err, format!("Failed to write storage file \"{}\"", self.file_path.display()).as_str())

        }
    }


}


impl Drop for Storage {

    fn drop(&mut self) {
        self.file.sync_all().unwrap_or_else(
            |err| error::io_error(&self.file_path, &err, format!("Failed to sync storage file \"{}\"", self.file_path.display()).as_str())
        );
    }

}


#[cfg(test)]
mod tests {

    use super::*;

    static LAST_UNIQUE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn get_unique_file_path() -> String {
        let id = LAST_UNIQUE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("test/test_storage_{}.disk", id)
    }
    

    #[test]
    fn test_create_storage() {
        let file = get_unique_file_path();
        let storage = Storage::new(PathBuf::from(&file), None);
        assert_eq!(storage.file_path, PathBuf::from(&file).canonicalize().unwrap());
    }


    #[test]
    fn test_read_write() {
        let storage = Storage::new(PathBuf::from(get_unique_file_path()), None);
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        storage.write(0, &data).unwrap();
        assert_eq!(storage.read(0, 8).unwrap(), data);

    }


    #[test]
    fn test_read_write_offset() {
        let storage = Storage::new(PathBuf::from(get_unique_file_path()), None);
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        storage.write(0, &data).unwrap();
        assert_eq!(storage.read(4, 4).unwrap(), vec![4, 5, 6, 7]);
    }


    #[test]
    fn test_read_write_offset_out_of_bounds() {
        let storage = Storage::new(PathBuf::from(get_unique_file_path()), None);
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        storage.write(0, &data).unwrap();
        assert!(matches!(storage.read(8, 4).err().unwrap(), ErrorCodes::EndOfFile));
    }


    #[test]
    fn test_write_overflow() {
        let storage = Storage::new(PathBuf::from(get_unique_file_path()), Some(8));
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let res = storage.write(0, &data);
        assert!(matches!(res.err().unwrap(), ErrorCodes::OutOfBounds));
    }


    #[test]
    fn test_read_write_max_size_exact() {
        let storage = Storage::new(PathBuf::from(get_unique_file_path()), Some(8));
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        storage.write(0, &data).unwrap();
        assert_eq!(storage.read(0, 8).unwrap(), data);
    }

}

