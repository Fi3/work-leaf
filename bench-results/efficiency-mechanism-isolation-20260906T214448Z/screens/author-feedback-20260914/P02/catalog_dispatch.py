"""Benchmark-only native command dispatch; retain the pinned P01 startup view."""
import importlib.util
from pathlib import Path
import sys

source = Path(__file__).resolve().parents[1] / "P01/private_catalog.py"
spec = importlib.util.spec_from_file_location("qualified_p01_catalog", source)
qualified = importlib.util.module_from_spec(spec)
spec.loader.exec_module(qualified)
validate_original = qualified.needs_view

VALUE_OPTIONS = frozenset({
    "--cd", "-C", "--model", "-m", "--ask-for-approval", "-a", "--sandbox", "-s",
    "--config", "-c", "--color", "--output-last-message", "-o", "--output-schema",
    "--profile", "-p", "--enable", "--disable", "--thread-source",
})
FLAG_OPTIONS = frozenset({
    "--json", "--strict-config", "--skip-git-repo-check", "--ephemeral",
    "--ignore-user-config", "--ignore-rules",
})


def positional(argv, offset):
    """Return the first positional and whether it follows a literal separator."""
    while offset < len(argv):
        item = argv[offset]
        if item == "--":
            return offset + 1, True
        if item == "-" or not item.startswith("-"):
            return offset, False
        key, equal, value = item.partition("=")
        if key in VALUE_OPTIONS:
            if equal:
                if not value:
                    raise ValueError("empty benchmark exec option")
                offset += 1
            else:
                if offset + 1 >= len(argv):
                    raise ValueError("incomplete benchmark exec option")
                offset += 2
        elif key in FLAG_OPTIONS and not equal:
            offset += 1
        else:
            raise ValueError("unsupported benchmark exec option")
    return offset, False


def needs_view(argv, env):
    validate_original(argv, env)  # Preserve opt-in and recursive-provider guards.
    command, escaped = positional(argv, 0)
    if escaped or command >= len(argv) or argv[command] != "exec":
        raise ValueError("only native exec is supported")
    subcommand, literal = positional(argv, command + 1)
    if literal or subcommand == len(argv):
        return True
    if argv[subcommand] == "resume":
        return False
    if argv[subcommand] in {"fork", "review", "help"}:
        raise ValueError("unsupported benchmark exec subcommand")
    return True


def main():
    qualified.needs_view = needs_view
    qualified.main()


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(86)
