use crate::terminal::Terminal;
use crate::storage::Storage;
use crate::host_fs::HostFS;



pub struct CPUModules {

    pub storage: Option<Storage>,
    pub terminal: Terminal,
    pub host_fs: HostFS

}


impl CPUModules {

    pub fn new(storage: Option<Storage>, terminal: Terminal, host_fs: HostFS) -> Self {
        Self {
            storage,
            terminal,
            host_fs,
        }
    }

}

