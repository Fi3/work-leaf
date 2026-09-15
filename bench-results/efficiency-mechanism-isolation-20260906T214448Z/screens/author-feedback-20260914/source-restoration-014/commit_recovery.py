"""Recover a retained Git commit only when every byte yields its known SHA."""
import hashlib
import re


def recover(*, tree, parent, message, identity, start, end, timezone, target):
    if not all(re.fullmatch(r"[0-9a-f]{40}", value)
               for value in (tree, parent, target)):
        raise ValueError("invalid Git identity")
    if (type(start) is not int or type(end) is not int or start > end
            or end - start > 86400 or start < 0):
        raise ValueError("timestamp search must be bounded to one day")
    if "\n" in identity or "\r" in identity or not identity:
        raise ValueError("invalid retained author identity")
    if not re.fullmatch(r"[+-]\d{4}", timezone):
        raise ValueError("invalid timezone")
    for timestamp in range(start, end + 1):
        raw = (f"tree {tree}\nparent {parent}\n"
               f"author {identity} {timestamp} {timezone}\n"
               f"committer {identity} {timestamp} {timezone}\n\n"
               f"{message}").encode("utf-8")
        digest = hashlib.sha1(f"commit {len(raw)}\0".encode() + raw).hexdigest()
        if digest == target:
            return raw, timestamp
    raise ValueError("no byte-exact original commit in the declared interval")
