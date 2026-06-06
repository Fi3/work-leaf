use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectInstructionFile {
    pub path: PathBuf,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RequiredCheck {
    command_line: String,
    program: String,
    args: Vec<String>,
}

impl RequiredCheck {
    pub(crate) fn command_line(&self) -> &str {
        &self.command_line
    }

    pub(crate) fn program(&self) -> &str {
        &self.program
    }

    pub(crate) fn args(&self) -> &[String] {
        &self.args
    }

    fn is_cargo_test(&self) -> bool {
        self.program == "cargo" && self.args.first().is_some_and(|arg| arg == "test")
    }

    fn parse(command_line: &str) -> Option<Self> {
        let command_line = command_line.trim();
        let parts = command_line
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let program = parts.first()?.clone();
        Some(Self {
            command_line: command_line.to_string(),
            program,
            args: parts.into_iter().skip(1).collect(),
        })
    }
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

pub(crate) fn required_checks(files: &[ProjectInstructionFile]) -> Vec<RequiredCheck> {
    let mut commands = Vec::new();
    let mut seen = BTreeSet::new();

    for file in files {
        for line in required_check_lines(&file.text) {
            for command_line in inline_code_spans(line) {
                let Some(command) = RequiredCheck::parse(command_line) else {
                    continue;
                };
                push_unique_check(&mut commands, &mut seen, command);
            }
        }
    }

    commands
}

pub(crate) fn validation_checks(
    root: &Path,
    files: &[ProjectInstructionFile],
) -> Vec<RequiredCheck> {
    let mut commands = required_checks(files);
    let mut seen = commands
        .iter()
        .map(|command| command.command_line.clone())
        .collect::<BTreeSet<_>>();

    if root.join("Cargo.toml").is_file() && !commands.iter().any(RequiredCheck::is_cargo_test) {
        let command = RequiredCheck::parse("cargo test --all-targets --all-features")
            .expect("built-in cargo test command is valid");
        push_unique_check(&mut commands, &mut seen, command);
    }

    commands
}

fn push_unique_check(
    commands: &mut Vec<RequiredCheck>,
    seen: &mut BTreeSet<String>,
    command: RequiredCheck,
) {
    if seen.insert(command.command_line.clone()) {
        commands.push(command);
    }
}

fn required_check_lines(text: &str) -> impl Iterator<Item = &str> {
    let mut in_section = false;
    text.lines().filter(move |line| {
        if is_required_checks_heading(line) {
            in_section = true;
            return false;
        }
        if in_section && is_heading(line) {
            in_section = false;
        }
        in_section
    })
}

fn is_required_checks_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix('#') else {
        return false;
    };
    let heading = rest.trim_start_matches('#').trim();
    heading.eq_ignore_ascii_case("Required Checks")
}

fn is_heading(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

fn inline_code_spans(line: &str) -> Vec<&str> {
    let mut spans = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        spans.push(&after_start[..end]);
        rest = &after_start[end + 1..];
    }
    spans
}
