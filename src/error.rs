use std::result;
use thiserror::Error;

#[derive(Debug, Error)]
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
}

pub type Result<T> = result::Result<T, Errors>;
