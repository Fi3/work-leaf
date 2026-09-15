"""Benchmark-only same-value config-layer permission qualification."""
import integration_probe as probe

_original_command = probe.command

def command(provider, repo, thread):
    original = _original_command(provider, repo, thread)
    return [original[0], "-c", 'sandbox_mode="danger-full-access"',
            "-c", 'approval_policy="never"', *original[1:]]

if __name__ == "__main__":
    probe.command = command
    probe.main()
