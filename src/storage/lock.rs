use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
/// File locking mechanism for concurrent access protection
///
/// This module provides a simple file-based locking mechanism to prevent
/// concurrent modifications to ticket files.
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Maximum time a lock can be held before it's considered stale
const LOCK_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum number of retry attempts for acquiring a lock
const MAX_RETRY_ATTEMPTS: u32 = 10;

/// Delay between retry attempts
const RETRY_DELAY: Duration = Duration::from_millis(100);

/// Information stored in a lock file
#[derive(Debug, Serialize, Deserialize)]
pub struct LockInfo {
    /// Unique identifier for the lock holder
    pub(crate) holder_id: String,
    /// Process ID of the lock holder
    pub(crate) pid: u32,
    /// Timestamp when the lock was acquired
    pub(crate) acquired_at: u64,
    /// Optional description of the operation
    pub(crate) operation: Option<String>,
}

/// A file lock that automatically releases on drop
pub struct FileLock {
    path: PathBuf,
    holder_id: String,
}

impl FileLock {
    /// Attempts to acquire a lock for the given path
    ///
    /// # Arguments
    /// * `path` - The file path to lock
    /// * `operation` - Optional description of the operation being performed
    ///
    /// # Returns
    /// A `FileLock` that will automatically release when dropped
    pub fn acquire(path: &Path, operation: Option<String>) -> Result<Self> {
        let lock_path = Self::lock_path(path);
        let holder_id = Uuid::new_v4().to_string();

        // Try to acquire the lock with retries
        for attempt in 0..MAX_RETRY_ATTEMPTS {
            match Self::try_acquire_once(&lock_path, &holder_id, &operation) {
                Ok(()) => {
                    return Ok(Self {
                        path: lock_path,
                        holder_id,
                    });
                },
                Err(e) => {
                    if attempt == MAX_RETRY_ATTEMPTS - 1 {
                        return Err(e).context("Failed to acquire lock after maximum retries");
                    }

                    // Check if the existing lock is stale
                    if Self::is_lock_stale(&lock_path)? {
                        // Try to remove stale lock
                        let _ = fs::remove_file(&lock_path);
                        continue;
                    }

                    // Wait before retrying
                    std::thread::sleep(RETRY_DELAY);
                },
            }
        }

        bail!("Failed to acquire lock: maximum retries exceeded")
    }

    /// Attempts to acquire a lock once without retrying
    fn try_acquire_once(
        lock_path: &Path,
        holder_id: &str,
        operation: &Option<String>,
    ) -> Result<()> {
        // Try to create the lock file exclusively
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
            .context("Lock file already exists")?;

        let lock_info = LockInfo {
            holder_id: holder_id.to_string(),
            pid: std::process::id(),
            acquired_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            operation: operation.clone(),
        };

        let json = serde_json::to_string_pretty(&lock_info)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;

        Ok(())
    }

    /// Checks if a lock file is stale (older than `LOCK_TIMEOUT`)
    fn is_lock_stale(lock_path: &Path) -> Result<bool> {
        if !lock_path.exists() {
            return Ok(false);
        }

        let mut file = File::open(lock_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let lock_info: LockInfo = serde_json::from_str(&contents)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let age = now.saturating_sub(lock_info.acquired_at);
        Ok(age > LOCK_TIMEOUT.as_secs())
    }

    /// Gets the lock file path for a given file
    fn lock_path(path: &Path) -> PathBuf {
        let mut lock_path = path.to_path_buf();
        let filename = format!("{}.lock", path.file_name().unwrap().to_str().unwrap());
        lock_path.set_file_name(filename);
        lock_path
    }

    /// Releases the lock explicitly
    pub fn release(self) {
        // Drop will handle the release
        drop(self);
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Only remove the lock file if we are the holder
        if let Ok(mut file) = File::open(&self.path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(lock_info) = serde_json::from_str::<LockInfo>(&contents) {
                    #[allow(clippy::excessive_nesting)]
                    if lock_info.holder_id == self.holder_id {
                        let _ = fs::remove_file(&self.path);
                    }
                }
            }
        }
    }
}

/// A guard that holds a lock and automatically releases it when dropped
pub struct LockGuard<'a> {
    _lock: FileLock,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl LockGuard<'_> {
    /// Creates a new lock guard
    #[must_use]
    pub const fn new(lock: FileLock) -> Self {
        LockGuard {
            _lock: lock,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_acquire_and_release_lock() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.yaml");

        // Acquire lock
        let lock = FileLock::acquire(&file_path, Some("test operation".to_string())).unwrap();

        // Try to acquire again - should fail
        assert!(FileLock::acquire(&file_path, None).is_err());

        // Release lock
        drop(lock);

        // Should be able to acquire again
        let _lock2 = FileLock::acquire(&file_path, None).unwrap();
    }

    #[test]
    fn test_stale_lock_removal() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.yaml");
        let lock_path = FileLock::lock_path(&file_path);

        // Create a stale lock manually
        let stale_info = LockInfo {
            holder_id: "stale-holder".to_string(),
            pid: 99999,
            acquired_at: 0, // Very old timestamp
            operation: None,
        };

        fs::write(&lock_path, serde_json::to_string(&stale_info).unwrap()).unwrap();

        // Should be able to acquire lock despite stale lock file
        let _lock = FileLock::acquire(&file_path, None).unwrap();
    }
}
