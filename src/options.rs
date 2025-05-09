use std::path::PathBuf;

#[derive(Clone)]
pub struct Options {
    // 数据库目录
    pub dir_path: PathBuf,

    // 数据文件大小
    pub data_file_size: u64,

    // 是否每次写入持久化
    pub sync_write: bool,

    // 索引类型
    pub index_type: IndexType,
}

#[derive(Clone)]
pub enum IndexType {
    /// BTree 索引
    BTree,

    /// 跳表 索引
    SkipList,
}
