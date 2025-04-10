use std::result;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Errors {
    #[error("failed to read from data file")]
    FailedToReadFromDataFile,

    #[error("failed to write to data file")]
    FailedToWriteToDataFile,

    #[error("failed to sync data file")]
    FailedToSyncDataFile,

    #[error("failed to open data file")]
    FailedToOpenDataFile,

    #[error("key is empty")]
    KeyIsEmpty,

    #[error("index update failed")]
    IndexUpdateFailed,

    #[error("key not found")]
    KeyNotFound,

    #[error("data file not found")]
    DataFileNotFound,

    #[error("database dir path is empty")]
    DirPathIsEmpty,

    #[error("database data file size is too small")]
    DataFileSizeTooSmall,

    #[error("failed to create database directory")]
    FailedToCreateDatabaseDir,

    #[error("failed to read database directory")]
    FailedToReadDatabaseDir,

    #[error("the database directory may be corrupted")]
    DataDirectoryCorrupted,

    #[error("read data file eof")]
    ReadDataFileEOF,
}

pub type Result<T> = result::Result<T, Errors>;
