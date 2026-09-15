"""Opt-in diagnostic: inherit persisted permission settings on native resume."""
import integration_probe as probe

original_command = probe.command


def command(provider, repo, thread):
    argv = original_command(provider, repo, thread)
    if thread is not None:
        for option, expected in (("--sandbox", "danger-full-access"),
                                 ("--ask-for-approval", "never")):
            index = argv.index(option)
            if argv[index + 1] != expected:
                raise ValueError("unexpected permission override")
            del argv[index:index + 2]
    return argv


if __name__ == "__main__":
    probe.command = command
    probe.main()
