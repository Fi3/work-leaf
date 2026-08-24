use std::env;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    if env::args().nth(1).as_deref() == Some("--version") {
        println!("bench-observer-fixture 1.0");
        return Ok(());
    }

    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    if let Some(stdout) = env::var_os("WORK_LEAF_OBSERVER_FIXTURE_STDOUT") {
        input = stdout.to_string_lossy().as_bytes().to_vec();
    }
    for chunk in input.chunks(5) {
        io::stdout().write_all(chunk)?;
        io::stdout().flush()?;
    }
    if let Some(stderr) = env::var_os("WORK_LEAF_OBSERVER_FIXTURE_STDERR") {
        io::stderr().write_all(stderr.to_string_lossy().as_bytes())?;
        io::stderr().flush()?;
    }
    if let Ok(code) = env::var("WORK_LEAF_OBSERVER_FIXTURE_EXIT_CODE")
        && let Ok(code) = code.parse::<i32>()
    {
        std::process::exit(code);
    }
    Ok(())
}
