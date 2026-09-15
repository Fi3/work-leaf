"""CLI-safe exact-project trust value; no quoted dotted configuration key."""
import json

import integration_probe as probe
import trust_probe

_original_command = probe.command


def command(provider, repo, thread):
    trust_probe.trust_override(repo)  # Reuse the exact-path rejection rules.
    original = _original_command(provider, repo, thread)
    override = 'projects={' + json.dumps(str(repo), ensure_ascii=False)
    override += '={trust_level="trusted"}}'
    return [original[0], '-c', override, *original[1:]]


if __name__ == '__main__':
    probe.command = command
    probe.main()
