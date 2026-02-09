use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub fn cwd_mutex() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    pub fn set(path: &Path) -> Self {
        let original = env::current_dir().expect("Failed to capture current directory");
        env::set_current_dir(path).expect("Failed to switch current directory for test");
        Self { original }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        env::set_current_dir(&self.original).expect("Failed to restore current directory");
    }
}
