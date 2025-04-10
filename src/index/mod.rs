mod btree;

use crate::{LogRecordPos, options::IndexType};

pub use btree::BTree;

/// Indexer 抽象索引接口
pub trait Indexer: Sync + Send {
    /// 向索引中存储 key 对应数据位置信息
    fn put(&self, key: Vec<u8>, pos: LogRecordPos) -> bool;

    /// 根据 key 取出对应的索引位置信息
    fn get(&self, key: Vec<u8>) -> Option<LogRecordPos>;

    /// 根据 key 删除对应的索引位置信息
    fn delete(&self, key: Vec<u8>) -> bool;
}

/// 根据索引类型打开内存索引
pub fn new_indexer(index_type: IndexType) -> impl Indexer {
    match index_type {
        IndexType::BTree => btree::BTree::default(),
        IndexType::SkipList => todo!(),
    }
}
