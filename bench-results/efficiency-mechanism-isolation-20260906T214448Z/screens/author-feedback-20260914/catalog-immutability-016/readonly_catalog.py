"""Immutable private catalog/config mounts; original caller namespace is untouched."""
import hashlib
from pathlib import Path
import types

ORIGINAL = Path(__file__).resolve().parent.parent / 'P01/private_catalog.py'
EXPECTED = 'a06cb222718aa85b3fd8bc2ff740e2062e35b4345d294930e40120fe79b46c54'
INSERTION = (
    '    for _, target in view["mounts"]:\n'
    '        if lib.mount(None, target.encode(), None, 4096 | 32 | 1, None) != 0:\n'
    '            raise OSError(ctypes.get_errno(), "private catalog read-only remount failed")\n'
)


def original_source():
    raw = ORIGINAL.read_bytes()
    if hashlib.sha256(raw).hexdigest() != EXPECTED:
        raise ValueError('retained catalog implementation changed')
    return raw.decode()


def source():
    raw = original_source()
    anchor = '    if os.getuid() != view["uid"]:\n'
    if raw.count(anchor) != 1:
        raise ValueError('retained capability boundary changed')
    return raw.replace(anchor, INSERTION + anchor)


def load():
    module = types.ModuleType('private_immutable_catalog')
    module.__file__ = str(ORIGINAL)
    exec(compile(source(), str(ORIGINAL), 'exec'), module.__dict__)
    return module
