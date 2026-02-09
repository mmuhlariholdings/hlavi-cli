pub mod agent;
pub mod board;
pub mod init;
pub mod sort;
pub mod tasks;
pub mod timeline;

use hlavi_core::storage::file_storage::FileStorage;
use std::env;
use std::path::PathBuf;

/// Gets the current working directory
pub fn get_cwd() -> anyhow::Result<PathBuf> {
    env::current_dir().map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))
}

/// Creates a storage instance for the current directory
pub fn get_storage() -> anyhow::Result<FileStorage> {
    let cwd = get_cwd()?;
    Ok(FileStorage::new(cwd))
}
