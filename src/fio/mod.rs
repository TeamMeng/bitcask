mod file_io;

use crate::Result;

pub use file_io::FileIo;

/// Io 管理接口, 可以插入不同的 IO 类型, 目前支持标准文件 IO
pub trait IoManger: Sync + Send {
    /// 从文件的给定位置读取对应的数据
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize>;

    /// 写入字节数据到文件中
    fn write(&self, buf: &[u8]) -> Result<usize>;

    /// 持久化数据
    fn sync(&self) -> Result<()>;
}
