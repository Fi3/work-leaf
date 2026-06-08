use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectInstructionFile {
    pub path: PathBuf,
    pub text: String,
}

pub(crate) fn load_project_instructions(root: &Path) -> io::Result<Vec<ProjectInstructionFile>> {
    let mut files = Vec::new();
    for name in ["AGENTS.md", "agent.md", "AGENT.md", "agents.md"] {
        let path = root.join(name);
        match fs::read_to_string(&path) {
            Ok(text) => files.push(ProjectInstructionFile {
                path: PathBuf::from(name),
                text,
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(files)
}
