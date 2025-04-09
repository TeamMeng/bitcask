use super::Indexer;
use crate::LogRecordPos;
use parking_lot::RwLock;
use std::{collections::BTreeMap, sync::Arc};

// BTree 索引, 主要封装标准库中的 BTreeMap 结构
#[derive(Default)]
pub struct BTree {
    tree: Arc<RwLock<BTreeMap<Vec<u8>, LogRecordPos>>>,
}

impl Indexer for BTree {
    fn get(&self, key: Vec<u8>) -> Option<LogRecordPos> {
        self.tree.read().get(&key).copied()
    }

    fn put(&self, key: Vec<u8>, pos: LogRecordPos) -> bool {
        if self.get(key.clone()).is_some() {
            return false;
        }

        let mut w = self.tree.write();
        w.insert(key, pos);
        true
    }

    fn delete(&self, key: Vec<u8>) -> bool {
        let mut w = self.tree.write();
        let ret = w.remove(&key);
        ret.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btree_get_put_delete_should_work() {
        let tree = BTree::default();

        let key = "".as_bytes().to_vec();

        // success put
        assert!(tree.put(key.clone(), LogRecordPos::new(1, 10)));

        // failed put
        assert!(!tree.put(key.clone(), LogRecordPos::new(1, 10)));

        let pos = tree.get(key.clone());

        // success get
        assert!(pos.is_some());

        assert_eq!(pos.unwrap().get_file_id(), 1);
        assert_eq!(pos.unwrap().get_offset(), 10);

        // success delete
        assert!(tree.delete(key.clone()));

        // fail delete
        assert!(!tree.delete("None".as_bytes().to_vec()));

        // fail get
        assert!(tree.get("None".as_bytes().to_vec()).is_none());
    }
}
