"""Prototype byte-level source oracle for a predeclared, sources-only snapshot.

No Texloom planner is imported. This checks declared bytes and one fresh object
ID, not TeX semantics. A separately authored rendered reference is still needed.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re

TOKEN = "{{NEW_OBJECT_ID}}"
HASH = re.compile(r"[0-9a-f]{64}\Z")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def valid_path(name):
    path = PurePosixPath(name)
    if (not name or not path.parts or path.is_absolute() or any(x in {".", ".."} for x in path.parts)
            or any(x in name for x in ["\\", ":", "\0"]) or path.as_posix() != name):
        raise ValueError("Expected a normalized relative source path")


def verify_sources(baseline, candidate, contract):
    result = {"schema": 1, "decision": "unresolved", "source_contract_checked": False,
              "tex_semantics_verified": False, "bound_object_id": None}
    try:
        if contract.get("schema") != 1 or not isinstance(contract.get("files"), dict):
            raise ValueError("Unsupported source contract")
        files = contract["files"]
        if not 1 <= len(files) <= 256 or len({x.casefold() for x in files}) != len(files):
            raise ValueError("Source contract has invalid or case-colliding paths")
        for name, rule in files.items():
            valid_path(name)
            if not isinstance(rule, dict) or "before_sha256" not in rule:
                raise ValueError("Missing declared baseline fingerprint")
            before = rule["before_sha256"]
            if before is not None and not (isinstance(before, str) and HASH.fullmatch(before)):
                raise ValueError("Malformed baseline fingerprint")
            if ("after_sha256" in rule) == ("after_utf8_template" in rule):
                raise ValueError("Declare exactly one expected hash or UTF-8 template")
            if "after_sha256" in rule and not (isinstance(rule["after_sha256"], str) and HASH.fullmatch(rule["after_sha256"])):
                raise ValueError("Malformed expected fingerprint")
            if "after_utf8_template" in rule and not isinstance(rule["after_utf8_template"], str):
                raise ValueError("Expected a UTF-8 template string")
        before_files = {name for name, rule in files.items() if rule["before_sha256"] is not None}
        if set(baseline) != before_files:
            raise ValueError("Baseline file set differs from the predeclared contract")
        if any(sha(baseline[name]) != files[name]["before_sha256"] for name in before_files):
            raise ValueError("Baseline bytes differ from the predeclared contract")
        if set(candidate) != set(files):
            return {**result, "decision": "differs", "reason": "Unexpected or missing candidate source files"}
        for name, rule in files.items():
            actual = candidate[name]
            if "after_sha256" in rule:
                matches = sha(actual) == rule["after_sha256"]
            else:
                template = rule["after_utf8_template"]
                if len(template.encode("utf-8")) > 16_000_000 or template.count(TOKEN) > 64:
                    raise ValueError("Source template exceeds research budget")
                if result["bound_object_id"] is not None:
                    matches = actual == template.replace(TOKEN, result["bound_object_id"]).encode("utf-8")
                else:
                    pieces = template.encode("utf-8").split(TOKEN.encode("ascii"))
                    pattern = re.escape(pieces[0])
                    for index, piece in enumerate(pieces[1:]):
                        pattern += (rb"(?P<object_id>text-[0-9a-f]{8})" if index == 0 else rb"(?P=object_id)")+re.escape(piece)
                    match = re.fullmatch(pattern, actual)
                    matches = match is not None
                    if matches and len(pieces) > 1:
                        result["bound_object_id"] = match["object_id"].decode("ascii")
            if not matches:
                return {**result, "decision": "differs", "reason": f"Source bytes violate declared change: {name}"}
        if result["bound_object_id"] is not None and any(
                result["bound_object_id"].encode("ascii") in data for data in baseline.values()):
            return {**result, "decision": "differs", "reason": "New object ID already occurs in baseline bytes"}
        return {**result, "decision": "matches_contract", "source_contract_checked": True,
                "file_count": len(files)}
    except (ValueError, TypeError, KeyError) as error:
        return {**result, "reason": str(error)}


def snapshot(root):
    root = root.resolve()
    files, total = {}, 0
    for path in root.rglob("*"):
        if path.is_symlink() or getattr(path.stat(), "st_file_attributes", 0) & 0x400:
            raise ValueError("Links/reparse points are outside this snapshot oracle")
        if not path.is_file():
            continue
        size = path.stat().st_size
        total += size
        if size > 16_000_000 or total > 64_000_000 or len(files) >= 256:
            raise ValueError("Source snapshot exceeds research budget")
        files[path.relative_to(root).as_posix()] = path.read_bytes()
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("contract", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    raw = args.contract.read_bytes()
    if len(raw) > 16_000_000:
        raise ValueError("Contract exceeds byte budget")
    result = verify_sources(snapshot(args.baseline), snapshot(args.candidate), json.loads(raw))
    result["contract_sha256"] = sha(raw)
    result["oracle_sha256"] = sha(Path(__file__).read_bytes())
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2);stream.write("\n")
    print(json.dumps(result))
    return 0 if result["decision"] == "matches_contract" else 1


if __name__ == "__main__":
    raise SystemExit(main())
