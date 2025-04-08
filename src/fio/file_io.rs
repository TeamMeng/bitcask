use super::IoManger;
use crate::{Errors, Result};
use log::error;
use parking_lot::RwLock;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::unix::fs::FileExt,
    path::PathBuf,
    sync::Arc,
};

pub struct FileIo {
    fd: Arc<RwLock<File>>,
}

impl FileIo {
    pub fn try_new(path: PathBuf) -> Result<Self> {
        match OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
        {
            Ok(file) => Ok(Self::new(file)),
            Err(e) => {
                error!("failed to open data file: {}", e);
                Err(Errors::FailedToOpenDataFile)
            }
        }
    }

    fn new(file: File) -> Self {
        Self {
            fd: Arc::new(RwLock::new(file)),
        }
    }
}

impl IoManger for FileIo {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let r = self.fd.read();
        match r.read_at(buf, offset) {
            Ok(n) => Ok(n),
            Err(e) => {
                error!("read from data file err: {}", e);
                Err(Errors::FailedToReadFromDataFile)
            }
        }
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        let mut w = self.fd.write();

        match w.write(buf) {
            Ok(n) => Ok(n),
            Err(e) => {
                error!("write to data file err: {}", e);
                Err(Errors::FailedToReadFromDataFile)
            }
        }
    }

    fn sync(&self) -> Result<()> {
        let r = self.fd.read();
        if let Err(e) = r.sync_all() {
            error!("failed to sync data file: {}", e);
            return Err(Errors::FailedToSyncDataFile);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn file_io_write_should_work() {
        let path = PathBuf::from("/tmp/a.data");

        let f = FileIo::try_new(path.clone());
        assert!(f.is_ok());

        let f = f.ok().unwrap();

        let ret = f.write("key-a".as_bytes());
        assert_eq!(ret.ok().unwrap(), 5);

        let res = fs::remove_file(path);
        assert!(res.is_ok());
    }

    #[test]
    fn file_io_read_should_work() {
        let path = PathBuf::from("/tmp/b.data");

        let f = FileIo::try_new(path.clone());
        assert!(f.is_ok());

        let f = f.ok().unwrap();

        f.write("key-a".as_bytes()).expect("write should work");

        let mut buf = vec![0u8; 5];
        let res = f.read(&mut buf, 0);

        assert_eq!(res.ok().unwrap(), 5);
        assert_eq!(buf, "key-a".as_bytes().to_vec());

        let res = fs::remove_file(path);
        assert!(res.is_ok());
    }

    #[test]
    fn file_io_fio_should_work() {
        let path = PathBuf::from("/tmp/c.data");

        let f = FileIo::try_new(path.clone());
        assert!(f.is_ok());

        let f = f.ok().unwrap();

        f.write("key-a".as_bytes()).expect("write should work");

        let mut buf = vec![0u8; 5];
        let res = f.read(&mut buf, 0);

        assert_eq!(res.ok().unwrap(), 5);
        assert_eq!(buf, "key-a".as_bytes().to_vec());

        let res = f.sync();

        assert!(res.is_ok());

        let res = fs::remove_file(path);
        assert!(res.is_ok());
    }
}
