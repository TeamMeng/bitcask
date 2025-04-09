use crate::{DataFile, Errors, Indexer, LogRecord, LogRecordPos, LogRecordType, Options, Result};
use bytes::Bytes;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

pub struct Engine {
    options: Arc<Options>,
    // 当前活跃数据文件
    active_file: Arc<RwLock<DataFile>>,
    // 旧的数据文件文件
    older_files: Arc<RwLock<HashMap<u32, DataFile>>>,
    // 数据内存索引
    index: Box<dyn Indexer>,
}

impl Engine {
    /// 存储 key/value 数据, key 不能为空
    pub fn put(&self, key: Bytes, value: Bytes) -> Result<()> {
        // 判断 key 的有效性
        if key.is_empty() {
            return Err(Errors::KeyIsEmpty);
        }

        // 构建 LogRecord
        let mut record = LogRecord::new(key.to_vec(), value.to_vec());

        // 追加写到活跃数据文件中
        let log_record_pos = self.append_log_record(&mut record)?;

        // 更新内存索引
        if !self.index.put(key.to_vec(), log_record_pos) {
            return Err(Errors::IndexUpdateFailed);
        }

        Ok(())
    }

    // 根据 key 获取对应的数据
    pub fn get(&self, key: Bytes) -> Result<Bytes> {
        if key.is_empty() {
            return Err(Errors::KeyIsEmpty);
        }

        // 从内存索引中获取 key 对应的数据信息
        let log_record = match self.index.get(key.to_vec()) {
            // 从对应的数据文件中获取对应的 LogRecord
            Some(pos) => {
                let active_file = self.active_file.read();
                let older_files = self.older_files.read();

                if active_file.get_file_id() == pos.get_file_id() {
                    active_file.read_log_record(pos.get_offset())?
                } else {
                    match older_files.get(&pos.get_file_id()) {
                        Some(data_file) => data_file.read_log_record(pos.get_offset())?,
                        None => return Err(Errors::DataFileNotFound),
                    }
                }
            }
            None => return Err(Errors::KeyNotFound),
        };

        // 判断 LogRecord 的类型
        if log_record.rec_type == LogRecordType::DELETED {
            return Err(Errors::KeyNotFound);
        }

        Ok(log_record.value.into())
    }

    // 追加写数据到当前活跃文件中
    fn append_log_record(&self, log_record: &mut LogRecord) -> Result<LogRecordPos> {
        let dir_path = self.options.dir_path.clone();

        // 输入数据进行编码
        let enc_record = log_record.encode();
        let record_len = enc_record.len() as u64;

        // 获取到当前活跃文件
        let mut active_file = self.active_file.write();
        // 判断当前活跃文件是否达到了阀值
        if active_file.get_write_off() + record_len > self.options.data_file_size {
            // 将当前活跃文件进行持久化
            active_file.sync()?;

            let current_fid = active_file.get_file_id();
            // 将旧的文件放入到 map 中
            let mut older_files = self.older_files.write();
            let old_file = DataFile::new(dir_path.clone(), current_fid)?;
            older_files.insert(current_fid, old_file);

            // 打开一个新的文件
            let new_file = DataFile::new(dir_path.clone(), current_fid + 1)?;
            *active_file = new_file;
        }

        // 追加写数据到当前活跃文件中
        let write_off = active_file.get_write_off();
        active_file.write(&enc_record)?;

        // 根据配置项决定是否持久化
        if self.options.sync_write {
            active_file.sync()?;
        }

        Ok(LogRecordPos::new(active_file.get_file_id(), write_off))
    }
}
