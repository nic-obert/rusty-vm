use crate::terminal::Terminal;
use crate::storage::Storage;



pub struct CPUModules {

    pub storage: Option<Storage>,
    pub terminal: Terminal

}


impl CPUModules {

    pub fn new(storage: Option<Storage>, terminal: Terminal) -> Self {
        Self {
            storage,
            terminal,
        }
    }

}

