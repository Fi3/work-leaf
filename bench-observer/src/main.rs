use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use work_leaf_bench_observer::{
    CONFIG_ENV, CaptureKind, InitSpec, ObserverError, ObserverResult, analyze,
    archive_context_bundles, capture_git_checkpoint, classify_codex_invocation,
    config_from_environment, enforce_cargo_validation_budget, exit_like_child,
    extract_rollout_metadata, initialize, is_locked_shell_invocation, pass_through,
    pass_through_cargo, pass_through_codex, record_controller_usage, record_timeline,
    run_captured_process, set_validation_budget, stop_active_primary_app_server,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("bench-observer: {error}");
        std::process::exit(1);
    }
}

fn run() -> ObserverResult<()> {
    let argv = env::args_os().collect::<Vec<_>>();
    let executable_name = argv
        .first()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("bench-observer");
    let args = argv.iter().skip(1).cloned().collect::<Vec<_>>();
    match executable_name {
        "codex" => run_codex_proxy(&args),
        "sh" => run_shell_proxy(&args),
        "cargo" => run_cargo_proxy(&args),
        _ => run_command(&args),
    }
}

fn run_cargo_proxy(args: &[OsString]) -> ObserverResult<()> {
    let config = config_from_environment()?;
    let real_cargo = config.real_cargo.as_deref().ok_or_else(|| {
        ObserverError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "observer configuration has no real Cargo executable",
        ))
    })?;
    enforce_cargo_validation_budget(&config, args)?;
    Err(pass_through_cargo(real_cargo, args))
}

fn run_codex_proxy(args: &[OsString]) -> ObserverResult<()> {
    let config = config_from_environment()?;
    let Some(kind) = classify_codex_invocation(args) else {
        return Err(pass_through_codex(&config, args));
    };
    let outcome = run_captured_process(
        &config,
        "codex",
        kind,
        &config.real_codex,
        &config.real_codex_sha256,
        args,
    )?;
    exit_like_child(outcome)
}

fn run_shell_proxy(args: &[OsString]) -> ObserverResult<()> {
    let config = config_from_environment()?;
    if !is_locked_shell_invocation(args) {
        return Err(pass_through(&config.real_sh, args));
    }
    let outcome = run_captured_process(
        &config,
        "sh",
        CaptureKind::LockedCommand,
        &config.real_sh,
        &config.real_sh_sha256,
        args,
    )?;
    exit_like_child(outcome)
}

fn run_command(args: &[OsString]) -> ObserverResult<()> {
    let Some(command) = args.first().and_then(|arg| arg.to_str()) else {
        return Err(ObserverError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            usage(),
        )));
    };
    let options = parse_options(&args[1..])?;
    match command {
        "init" => {
            let observer_executable = env::current_exe()?;
            let config = initialize(InitSpec {
                root: required_path(&options, "--root")?,
                study_id: required(&options, "--study-id")?,
                pair_id: required(&options, "--pair-id")?,
                condition: required(&options, "--condition")?,
                run_id: required(&options, "--run-id")?,
                real_codex: required_path(&options, "--real-codex")?,
                real_sh: required_path(&options, "--real-sh")?,
                real_cargo: required_path(&options, "--real-cargo")?,
                base_commit: required(&options, "--base-commit")?,
                experiment_commit: required(&options, "--experiment-commit")?,
                model: required(&options, "--model")?,
                effort: required(&options, "--effort")?,
                observer_executable,
            })?;
            println!("{}", config.path().display());
        }
        "analyze" => {
            let config = load_option_config(&options)?;
            let summary = analyze(&config)?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            if !summary.capture_complete {
                std::process::exit(2);
            }
        }
        "archive-bundles" => {
            let config = load_option_config(&options)?;
            let count = archive_context_bundles(&config, &required_path(&options, "--source")?)?;
            println!("{count}");
        }
        "timeline" => {
            let config = load_option_config(&options)?;
            let event = required(&options, "--event")?;
            let detail = options.get("--detail").map(String::as_str);
            record_timeline(&config, &event, detail)?;
        }
        "git-checkpoint" => {
            let config = load_option_config(&options)?;
            capture_git_checkpoint(
                &config,
                &required_path(&options, "--repo")?,
                &required(&options, "--label")?,
            )?;
        }
        "controller-state" => {
            let config = load_option_config(&options)?;
            let count = record_controller_usage(&config, &required_path(&options, "--state")?)?;
            println!("{count}");
        }
        "stop-app-server" => {
            let config = load_option_config(&options)?;
            let count = stop_active_primary_app_server(&config, Duration::from_secs(5))?;
            println!("{count}");
        }
        "validation-budget" => {
            let config = load_option_config(&options)?;
            let enabled = match required(&options, "--state")?.as_str() {
                "enabled" => true,
                "disabled" => false,
                state => {
                    return Err(ObserverError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("invalid validation budget state {state}"),
                    )));
                }
            };
            set_validation_budget(&config, enabled)?;
        }
        "extract-rollouts" => {
            let config = load_option_config(&options)?;
            let audit =
                extract_rollout_metadata(&config, &required_path(&options, "--sessions-root")?)?;
            println!("{}", serde_json::to_string_pretty(&audit)?);
            if !audit.errors.is_empty() {
                std::process::exit(2);
            }
        }
        _ => {
            return Err(ObserverError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                usage(),
            )));
        }
    }
    Ok(())
}

fn parse_options(args: &[OsString]) -> ObserverResult<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        return Err(ObserverError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "observer options require --name value pairs",
        )));
    }
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let name = pair[0].to_string_lossy().to_string();
        let value = pair[1].to_string_lossy().to_string();
        if !name.starts_with("--") {
            return Err(ObserverError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid observer option {name}"),
            )));
        }
        if options.insert(name.clone(), value).is_some() {
            return Err(ObserverError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("duplicate observer option {name}"),
            )));
        }
    }
    Ok(options)
}

fn required(options: &BTreeMap<String, String>, name: &str) -> ObserverResult<String> {
    options.get(name).cloned().ok_or_else(|| {
        ObserverError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("missing required option {name}"),
        ))
    })
}

fn required_path(options: &BTreeMap<String, String>, name: &str) -> ObserverResult<PathBuf> {
    required(options, name).map(PathBuf::from)
}

fn load_option_config(
    options: &BTreeMap<String, String>,
) -> ObserverResult<work_leaf_bench_observer::CaptureConfig> {
    let path = options
        .get("--config")
        .map(PathBuf::from)
        .or_else(|| env::var_os(CONFIG_ENV).map(PathBuf::from))
        .ok_or_else(|| {
            ObserverError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing --config and WORK_LEAF_OBSERVER_CONFIG",
            ))
        })?;
    work_leaf_bench_observer::CaptureConfig::load(&path)
}

fn usage() -> &'static str {
    "usage: bench-observer <init|analyze|archive-bundles|timeline|git-checkpoint|controller-state|stop-app-server|validation-budget|extract-rollouts> [--name value ...]"
}
