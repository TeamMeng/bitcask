#[derive(PartialEq, Clone)]
pub enum LogRecordType {
    // 正常 put 的数据
    NORMAL = 1,

    // 被删除的数据标记
    DELETED = 2,
}

/// LogRecord 写入到数据文件的记录
/// 数据文件中的数据水追加写入的, 类似日志的格式
#[derive(Clone)]
pub struct LogRecord {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub rec_type: LogRecordType,
}

// 数据位置索引信息, 描述数据存储到哪个位置
#[derive(Clone, Copy)]
pub struct LogRecordPos {
    file_id: u32,
    offset: u64,
}

/// 从数据文件中读取的 log_record 信息, 包含其 size
pub struct ReadLogRecord {
    pub(crate) record: LogRecord,
    pub(crate) size: u64,
}

impl LogRecordPos {
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

impl LogRecord {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            key,
            value,
            rec_type: LogRecordType::NORMAL,
        }
    }

    pub fn encode(&mut self) -> Vec<u8> {
        todo!()
    }
}
