use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CreatedLinkRecord {
    pub path: PathBuf,
    pub target: PathBuf,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct RemovedLinkRecord {
    pub path: PathBuf,
    pub target: PathBuf,
    pub kind: String,
}

/// RAII transaction guard for virtualization.
/// If commit() is not explicitly called, on drop it automatically reverses
/// any created links and attempts to restore removed links.
pub struct RollbackGuard {
    pub created_links: Vec<CreatedLinkRecord>,
    pub removed_links: Vec<RemovedLinkRecord>,
    committed: bool,
}

impl RollbackGuard {
    pub fn new() -> Self {
        Self {
            created_links: Vec::new(),
            removed_links: Vec::new(),
            committed: false,
        }
    }

    pub fn record_create(&mut self, path: PathBuf, target: PathBuf, kind: String) {
        self.created_links.push(CreatedLinkRecord { path, target, kind });
    }

    pub fn record_remove(&mut self, path: PathBuf, target: PathBuf, kind: String) {
        self.removed_links.push(RemovedLinkRecord { path, target, kind });
    }

    pub fn commit(mut self) {
        self.committed = true;
    }

    pub fn execute_rollback(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        // 1. Remove created links in reverse order
        for rec in self.created_links.iter().rev() {
            if rec.path.exists() || is_symlink_or_junction(&rec.path) {
                if let Err(e) = fs::remove_file(&rec.path) {
                    errors.push(format!("Failed to undo created link {}: {}", rec.path.display(), e));
                }
            }
        }

        // 2. Restore removed links
        for rec in &self.removed_links {
            if !rec.path.exists() && rec.target.is_file() {
                if let Err(e) = restore_reference(&rec.target, &rec.path, &rec.kind) {
                    errors.push(format!("Failed to restore link {}: {}", rec.path.display(), e));
                }
            }
        }

        errors
    }
}

impl Drop for RollbackGuard {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.execute_rollback();
        }
    }
}

fn is_symlink_or_junction(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn restore_reference(source: &Path, destination: &Path, kind: &str) -> std::io::Result<()> {
    if kind == "symlink" {
        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_file(source, destination).is_ok() {
                return Ok(());
            }
        }
        #[cfg(unix)]
        {
            if std::os::unix::fs::symlink(source, destination).is_ok() {
                return Ok(());
            }
        }
    }

    if kind == "hardlink" || kind == "symlink" {
        if fs::hard_link(source, destination).is_ok() {
            return Ok(());
        }
    }

    fs::copy(source, destination).map(|_| ())
}
