"""Bounded local research data I/O; synthetic infrastructure checks are recorded.

These checks help detect mistakes in controlled artifact directories. They are
not an OS sandbox against hostile concurrent filesystem changes.
"""
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import stat


MAX_JSON_BYTES = 50 * 1024 * 1024
MAX_FILE_BYTES = 128 * 1024 * 1024
MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024
MAX_FILES = 10000


def require(condition, message):
    if not condition:
        raise ValueError(message)


def relative_name(value):
    require(isinstance(value, str) and 0 < len(value) <= 1024 and
            not any(ord(char) < 32 or 0xd800 <= ord(char) <= 0xdfff or char in '\\:<>"|?*' for char in value) and
            not PurePosixPath(value).is_absolute(), "Invalid relative artifact path")
    parts = value.split("/")
    devices = {"CON", "PRN", "AUX", "NUL", *(f"COM{i}" for i in range(1, 10)), *(f"LPT{i}" for i in range(1, 10))}
    require(all(part not in {"", ".", ".."} and not part.endswith((".", " ")) and
                part.split(".", 1)[0].upper() not in devices for part in parts), "Nonportable artifact path")
    return value


def linked(path):
    info = path.lstat()
    return (stat.S_ISLNK(info.st_mode) or
            bool(getattr(info, "st_file_attributes", 0) & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0)))


def contained_file(root, name):
    relative_name(name)
    root = Path(root)
    require(not linked(root) and root.is_dir(), "Evidence root must be an existing ordinary directory")
    root = root.resolve(strict=True)
    path = root
    for part in name.split("/"):
        path /= part
        require(not linked(path), "Artifact paths must not contain links or reparse points")
    resolved = path.resolve(strict=True)
    require(resolved.is_relative_to(root) and resolved.is_file(), "Artifact must be an ordinary file inside its explicit root")
    return resolved


def identity(info):
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def read_fingerprint(path, *, retain=False, limit=MAX_FILE_BYTES):
    path = Path(path)
    require(not linked(path), "Linked artifact input")
    require(stat.S_ISREG(path.lstat().st_mode), "Artifact input must be an ordinary file")
    digest, chunks, count = hashlib.sha256(), [], 0
    with path.open("rb") as stream:
        before = os.fstat(stream.fileno())
        require(stat.S_ISREG(before.st_mode) and before.st_size <= limit, "Artifact type or size budget exceeded")
        while chunk := stream.read(1024 * 1024):
            count += len(chunk)
            require(count <= limit, "Artifact changed beyond its byte budget")
            digest.update(chunk)
            if retain:
                chunks.append(chunk)
        after = os.fstat(stream.fileno())
    path_after = path.stat()
    # Windows fstat/stat can expose different ctime semantics for the same
    # unchanged file. Compare ctime within the handle API, and stable file ID,
    # size and mtime across handle/path APIs. POSIX retains the full comparison.
    path_matches = (identity(after)[:4] == identity(path_after)[:4] if os.name == "nt"
                    else identity(after) == identity(path_after))
    require(identity(before) == identity(after) and path_matches and count == after.st_size,
            "Artifact changed during fingerprinting")
    return {"bytes": count, "sha256": digest.hexdigest()}, b"".join(chunks) if retain else None


def strict_json(raw):
    def object_pairs(pairs):
        value = {}
        for key, item in pairs:
            require(key not in value, "Duplicate JSON key")
            value[key] = item
        return value

    def constant(_value):
        raise ValueError("Nonfinite JSON number")

    def decimal(value):
        number = float(value)
        require(math.isfinite(number), "JSON number overflow")
        return number

    return json.loads(raw.decode("utf-8"), object_pairs_hook=object_pairs, parse_constant=constant, parse_float=decimal)


def read_json(path):
    receipt, raw = read_fingerprint(path, retain=True, limit=MAX_JSON_BYTES)
    return strict_json(raw), receipt


def write_json(path, value):
    # No creation of directories, overwrite, subprocess, extraction or network I/O.
    with Path(path).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write("\n")
