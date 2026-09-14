"""Local cancellation fixture; never starts an agent or touches the supplied root."""
import signal
import sys

signal.signal(signal.SIGINT, lambda *_: sys.exit(130))
print("ready", flush=True)
signal.pause()
