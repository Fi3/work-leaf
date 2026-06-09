#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once, OnceLock};

static REGISTER_AT_EXIT: Once = Once::new();
static TEMP_ROOTS: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();

pub fn register(path: &Path) {
    REGISTER_AT_EXIT.call_once(register_cleanup);
    TEMP_ROOTS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push(path.to_path_buf());
}

#[cfg(unix)]
fn register_cleanup() {
    unsafe {
        let _ = atexit(cleanup_temp_roots);
    }
}

#[cfg(not(unix))]
fn register_cleanup() {}

#[cfg(unix)]
unsafe extern "C" {
    fn atexit(callback: extern "C" fn()) -> i32;
}

#[cfg(unix)]
extern "C" fn cleanup_temp_roots() {
    let Some(roots) = TEMP_ROOTS.get() else {
        return;
    };
    let Ok(mut roots) = roots.lock() else {
        return;
    };
    for root in roots.drain(..) {
        let _ = fs::remove_dir_all(root);
    }
}
