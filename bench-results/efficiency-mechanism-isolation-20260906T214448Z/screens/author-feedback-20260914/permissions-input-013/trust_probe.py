"""Private exact-checkout trust qualification; preserve integration permissions."""
import json
from pathlib import Path

import integration_probe as probe

_original_command = probe.command


def trust_override(repo):
    raw = str(repo)
    path = Path(raw)
    if (not path.is_absolute() or len(path.parts) < 4 or '..' in path.parts
            or str(path) != raw or any(ord(c) < 32 for c in raw)):
        raise ValueError('an exact absolute checkout path is required')
    return 'projects.' + json.dumps(raw, ensure_ascii=False) + '.trust_level="trusted"'


def command(provider, repo, thread):
    override = trust_override(repo)
    original = _original_command(provider, repo, thread)
    return [original[0], '-c', override, *original[1:]]


if __name__ == '__main__':
    probe.command = command
    probe.main()
