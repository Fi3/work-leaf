use std::env;
use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};
use work_leaf::{AgentId, FileLockTable, GitPatcher, PatchRequest};

fn execute() -> Result<Value, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let root = PathBuf::from(args.next().ok_or("missing private root")?);
    let input = PathBuf::from(args.next().ok_or("missing held proposal")?);
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let value: Value = serde_json::from_slice(&fs::read(input)?)?;
    let text = |name: &str| -> Result<&str, Box<dyn std::error::Error>> {
        value[name]
            .as_str()
            .ok_or_else(|| format!("missing proposal field {name}").into())
    };
    let request = PatchRequest::new(
        AgentId::new(text("agent_id")?)?,
        text("feature")?,
        text("reason")?,
        text("body")?,
    );
    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root));
    let result = match text("format")? {
        "edit" => patcher.apply_edit(request),
        "patch" => patcher.apply(request),
        _ => return Err("unsupported held patch format".into()),
    };
    Ok(match result {
        Ok(outcome) => json!({
            "applied_privately": true,
            "private_commit": outcome.commit,
            "files": outcome.files,
            "shared_accepted": false
        }),
        Err(error) => json!({
            "applied_privately": false,
            "diagnostic": error.to_string(),
            "shared_accepted": false
        }),
    })
}

fn main() {
    match execute() {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("private patch driver failed: {error}");
            std::process::exit(1);
        }
    }
}
