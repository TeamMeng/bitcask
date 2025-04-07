// 数据位置索引信息, 描述数据存储到哪个位置
#[derive(Clone, Copy)]
pub struct LogRecord {
    file_id: u32,
    offset: u64,
}

impl LogRecord {
    pub fn new(file_id: u32, offset: u64) -> Self {
        Self { file_id, offset }
    }

    pub fn get_file_id(&self) -> u32 {
        self.file_id
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }
}
