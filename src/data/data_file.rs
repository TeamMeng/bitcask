use super::LogRecord;
use crate::{IoManger, Result};
use parking_lot::RwLock;
use std::{path::PathBuf, sync::Arc};

#[allow(unused)]
pub struct DataFile {
    // 数据文件id
    file_id: Arc<RwLock<u32>>,
    // 当前写偏移, 记录改数据文件写到哪个位置
    write_off: Arc<RwLock<u64>>,
    // IO 管理接口
    io_manager: Box<dyn IoManger>,
}

#[allow(unused)]
impl DataFile {
    pub fn new(dir_path: PathBuf, file_id: u32) -> Result<Self> {
        todo!()
    }

    pub fn get_write_off(&self) -> u64 {
        *self.write_off.read()
    }

    pub fn get_file_id(&self) -> u32 {
        *self.file_id.read()
    }

    pub fn read_log_record(&self, offset: u64) -> Result<LogRecord> {
        todo!()
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        todo!()
    }

    pub fn sync(&self) -> Result<()> {
        Ok(())
    }
}
