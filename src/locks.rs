use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone, Debug)]
pub struct FileLockTable {
    inner: Arc<FileLockTableInner>,
}

#[derive(Debug)]
struct FileLockTableInner {
    root: PathBuf,
    locks: Mutex<BTreeMap<PathBuf, Arc<RwLock<()>>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSnapshot {
    pub path: PathBuf,
    pub text: String,
}

impl FileLockTable {
    pub fn new(root: PathBuf) -> Self {
        Self {
            inner: Arc::new(FileLockTableInner {
                root,
                locks: Mutex::new(BTreeMap::new()),
            }),
        }
    }

    pub fn root(&self) -> &Path {
        &self.inner.root
    }

    pub fn read_files(&self, paths: &[PathBuf]) -> Result<Vec<FileSnapshot>, FileAccessError> {
        let normalized = self.normalize_paths(paths)?;
        self.with_read_locks(&normalized, || {
            normalized
                .iter()
                .map(|path| {
                    fs::read_to_string(self.inner.root.join(path))
                        .map(|text| FileSnapshot {
                            path: path.clone(),
                            text,
                        })
                        .map_err(FileAccessError::Io)
                })
                .collect()
        })
    }

    pub fn with_read_locks<F, T>(
        &self,
        paths: &[PathBuf],
        operation: F,
    ) -> Result<T, FileAccessError>
    where
        F: FnOnce() -> Result<T, FileAccessError>,
    {
        let locks = self.locks_for(paths)?;
        let mut guards = Vec::with_capacity(locks.len());
        for lock in &locks {
            guards.push(lock.read().map_err(|_| FileAccessError::Poisoned)?);
        }
        let result = operation();
        drop(guards);
        result
    }

    pub fn with_write_locks<F, T>(
        &self,
        paths: &[PathBuf],
        operation: F,
    ) -> Result<T, FileAccessError>
    where
        F: FnOnce() -> Result<T, FileAccessError>,
    {
        let locks = self.locks_for(paths)?;
        let mut guards = Vec::with_capacity(locks.len());
        for lock in &locks {
            guards.push(lock.write().map_err(|_| FileAccessError::Poisoned)?);
        }
        let result = operation();
        drop(guards);
        result
    }

    pub fn normalize_path(&self, path: &Path) -> Result<PathBuf, FileAccessError> {
        normalize_relative_path(path)
    }

    fn normalize_paths(&self, paths: &[PathBuf]) -> Result<Vec<PathBuf>, FileAccessError> {
        let mut normalized = paths
            .iter()
            .map(|path| self.normalize_path(path))
            .collect::<Result<Vec<_>, _>>()?;
        normalized.sort();
        normalized.dedup();
        Ok(normalized)
    }

    fn locks_for(&self, paths: &[PathBuf]) -> Result<Vec<Arc<RwLock<()>>>, FileAccessError> {
        let normalized = self.normalize_paths(paths)?;
        let mut map = self
            .inner
            .locks
            .lock()
            .map_err(|_| FileAccessError::Poisoned)?;
        let mut locks = Vec::with_capacity(normalized.len());
        for path in normalized {
            locks.push(
                map.entry(path)
                    .or_insert_with(|| Arc::new(RwLock::new(())))
                    .clone(),
            );
        }
        Ok(locks)
    }
}

fn normalize_relative_path(path: &Path) -> Result<PathBuf, FileAccessError> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(FileAccessError::PathEscapesRoot(path.to_path_buf()));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        Ok(PathBuf::from("."))
    } else {
        Ok(normalized)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandWriteIntent {
    pub writes: bool,
    pub paths: Vec<PathBuf>,
}

impl CommandWriteIntent {
    fn read_only() -> Self {
        Self {
            writes: false,
            paths: Vec::new(),
        }
    }

    fn writes(paths: &[&str]) -> Self {
        Self {
            writes: true,
            paths: paths.iter().map(PathBuf::from).collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CommandWritePolicy;

impl CommandWritePolicy {
    pub fn classify<I, S>(&self, command: I) -> CommandWriteIntent
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let parts = command
            .into_iter()
            .map(|part| part.as_ref().to_string())
            .collect::<Vec<_>>();
        let Some(program) = parts.first().map(String::as_str) else {
            return CommandWriteIntent::read_only();
        };
        let subcommand = parts.get(1).map(String::as_str);

        match (program, subcommand) {
            ("cargo", Some("build" | "check" | "clippy" | "doc" | "run" | "test" | "bench")) => {
                CommandWriteIntent::writes(&["target"])
            }
            ("cargo", Some("fmt" | "fix")) => CommandWriteIntent::writes(&["."]),
            ("rustc", _) => CommandWriteIntent::writes(&["."]),
            ("npm" | "pnpm" | "yarn", Some("install" | "add" | "update" | "remove")) => {
                CommandWriteIntent::writes(&["node_modules", "package-lock.json"])
            }
            ("npm" | "pnpm" | "yarn", Some("run" | "build" | "test")) => {
                CommandWriteIntent::writes(&["node_modules"])
            }
            ("go", Some("build" | "test" | "run")) => CommandWriteIntent::writes(&["."]),
            ("make" | "cmake", _) => CommandWriteIntent::writes(&["."]),
            ("pytest", _) => CommandWriteIntent::writes(&[".pytest_cache"]),
            ("python" | "python3", Some("-m")) if parts.get(2).is_some_and(|p| p == "pytest") => {
                CommandWriteIntent::writes(&[".pytest_cache"])
            }
            _ => CommandWriteIntent::read_only(),
        }
    }
}

#[derive(Debug)]
pub enum FileAccessError {
    PathEscapesRoot(PathBuf),
    Io(std::io::Error),
    Poisoned,
}

impl fmt::Display for FileAccessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathEscapesRoot(path) => {
                write!(formatter, "{} escapes project root", path.display())
            }
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Poisoned => formatter.write_str("file lock was poisoned"),
        }
    }
}

impl std::error::Error for FileAccessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FileAccessError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
