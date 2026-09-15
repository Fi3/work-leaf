"""Private full-driver observer-path qualification; original source is immutable."""
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "full-workflow-009"))
import full_workflow as original


def driver_source():
    source = original.driver_source()
    before = '  mkdir -p "$observer_root"\n'
    after = before + (
        '  local observer_identity\n'
        '  observer_identity="$(stat -Lc \'%d:%i\' -- "$observer_root")" '
        '|| fail_bench "observer directory identity unavailable"\n'
        '  observer_root="$(cd -- "$observer_root" && pwd -P)" '
        '|| fail_bench "observer canonical directory unavailable"\n'
        '  [[ "$(stat -Lc \'%d:%i\' -- "$observer_root")" == "$observer_identity" ]] '
        '|| fail_bench "observer canonical directory identity differs"\n'
    )
    return original.inverse.replace_once(source, before, after)


if __name__ == "__main__":
    print(driver_source(), end="")
