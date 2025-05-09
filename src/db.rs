use crate::{
    DATA_FILE_SUFFIX, DataFile, Errors, Indexer, LogRecord, LogRecordPos, LogRecordType, Options,
    Result, index,
};
use bytes::Bytes;
use log::warn;
use parking_lot::RwLock;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

const INITIAL_FILE_ID: u32 = 0;

pub struct Engine {
    options: Arc<Options>,
    // 当前活跃数据文件
    active_file: Arc<RwLock<DataFile>>,
    // 旧的数据文件文件
    older_files: Arc<RwLock<HashMap<u32, DataFile>>>,
    // 数据内存索引
    index: Box<dyn Indexer>,
    // 文件 id 信息
    file_ids: Vec<u32>,
}

impl Engine {
    // 打开 bitcask 存储引擎实例
    pub fn open(opts: Options) -> Result<Self> {
        // 考察配置项
        if let Some(e) = check_options(&opts) {
            return Err(e);
        }

        let options = opts.clone();
        // 判断数据目录是否存在, 如果不存在的话则创建这个目录
        let dir_path = options.dir_path.clone();
        if !dir_path.is_dir() {
            if let Err(e) = fs::create_dir_all(&dir_path) {
                warn!("create database directory failed: {}", e);
                return Err(Errors::FailedToCreateDatabaseDir);
            }
        }

        // 加载数据文件
        let mut data_files = load_data_files(dir_path.clone())?;

        // 设置 file id 信息
        let mut file_ids = Vec::new();
        for v in data_files.iter() {
            file_ids.push(v.get_file_id());
        }

        // 把老的数据文件保存到 older_files 中
        let mut older_files = HashMap::new();
        if data_files.len() > 1 {
            let mut idx = 0;
            let size = data_files.len();
            while let Some(file) = data_files.pop() {
                if idx < size - 1 {
                    older_files.insert(file.get_file_id(), file);
                } else {
                    break;
                }
                idx += 1;
            }
        }

        // 拿到当前活跃文件, 列表中最后一个文件
        let active_file = match data_files.pop() {
            Some(file) => file,
            None => DataFile::new(dir_path.clone(), INITIAL_FILE_ID)?,
        };

        // 构建存储引擎
        let mut engine = Engine {
            options: Arc::new(opts),
            active_file: Arc::new(RwLock::new(active_file)),
            older_files: Arc::new(RwLock::new(older_files)),
            index: Box::new(index::new_indexer(options.index_type)),
            file_ids,
        };

        engine.load_index_from_data_files()?;

        Ok(engine)
    }

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

    /// 根据 key 删除对应的数据
    pub fn delete(&self, key: Bytes) -> Result<()> {
        // 判断 key 的有效性
        if key.is_empty() {
            return Err(Errors::KeyIsEmpty);
        }

        // 从内存索引当中取出对应的数据, 不存在的话直接返回
        if self.index.get(key.to_vec()).is_none() {
            return Ok(());
        }

        // 构建 LogRecord, 标识算是被删除的
        let mut record = LogRecord {
            key: key.to_vec(),
            value: Default::default(),
            rec_type: LogRecordType::DELETED,
        };

        // 写入到数据文件中
        self.append_log_record(&mut record)?;

        // 更新内存索引中对应的 key
        let flag = self.index.delete(key.to_vec());
        if !flag {
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
                    active_file.read_log_record(pos.get_offset())?.record
                } else {
                    match older_files.get(&pos.get_file_id()) {
                        Some(data_file) => data_file.read_log_record(pos.get_offset())?.record,
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

    /// 遍历数据文件中的内容, 并依次处理其中的数据
    fn load_index_from_data_files(&mut self) -> Result<()> {
        // 数据文件为空, 直接返回
        if self.file_ids.is_empty() {
            return Ok(());
        }

        let active_file = self.active_file.read();
        let older_files = self.older_files.read();

        // 遍历每个文件 id, 取出对应的数据文件, 并加载其中的数据
        for (i, file_id) in self.file_ids.iter().enumerate() {
            let mut offset = 0;
            loop {
                let log_record_res = match *file_id == active_file.get_file_id() {
                    true => active_file.read_log_record(offset),
                    false => {
                        let data_file = older_files.get(file_id).unwrap();
                        data_file.read_log_record(offset)
                    }
                };

                let (log_record, size) = match log_record_res {
                    Ok(result) => (result.record, result.size),
                    Err(e) => {
                        if e == Errors::ReadDataFileEOF {
                            break;
                        }
                        return Err(e);
                    }
                };

                let log_record_pos = LogRecordPos::new(*file_id, offset);

                let flag = match log_record.rec_type {
                    LogRecordType::NORMAL => {
                        self.index.put(log_record.key.to_vec(), log_record_pos)
                    }
                    LogRecordType::DELETED => self.index.delete(log_record.key.to_vec()),
                };

                if !flag {
                    return Err(Errors::IndexUpdateFailed);
                }

                // 　递增 offset, 下一次读取的时候从新的位置开始
                offset += size;
            }

            // 设置活跃文件的 offset
            if i == self.file_ids.len() - 1 {
                active_file.set_write_off(offset);
            }
        }

        Ok(())
    }
}

// 从数据目录中加载数据文件
fn load_data_files(dir_path: PathBuf) -> Result<Vec<DataFile>> {
    match fs::read_dir(dir_path.clone()) {
        Ok(dir) => {
            let mut file_ids = Vec::new();
            let mut data_files = Vec::new();
            for entry in dir.into_iter().flatten() {
                // 拿到文件名
                if let Some(file_name) = entry.file_name().to_str() {
                    // 判断文件名后缀是否为 .data
                    if file_name.ends_with(DATA_FILE_SUFFIX) {
                        // 00001.data
                        let split_file_name: Vec<&str> = file_name.split(".").collect();
                        let file_id = match split_file_name[0].parse::<u32>() {
                            Ok(fid) => fid,
                            Err(_) => {
                                return Err(Errors::DataDirectoryCorrupted);
                            }
                        };
                        file_ids.push(file_id);
                    }
                }
            }

            // 如果没有数据文件, 则直接返回
            if file_ids.is_empty() {
                return Ok(data_files);
            }

            // 对文件 id 进行排序
            file_ids.sort_unstable();
            // 遍历文件id, 依次打开对应的数据文件
            for file_id in file_ids {
                data_files.push(DataFile::new(dir_path.clone(), file_id)?);
            }
            Ok(data_files)
        }
        Err(e) => {
            warn!("failed to read database directory: {}", e);
            Err(Errors::FailedToReadDatabaseDir)
        }
    }
}

fn check_options(opts: &Options) -> Option<Errors> {
    let dir_path = opts.dir_path.to_str();
    if let Some(size) = dir_path {
        if size.is_empty() {
            return Some(Errors::DirPathIsEmpty);
        }
    } else {
        return Some(Errors::DirPathIsEmpty);
    }

    if opts.data_file_size == 0 {
        return Some(Errors::DataFileSizeTooSmall);
    }

    None
}
