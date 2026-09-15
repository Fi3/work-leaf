"""Append-only population-check correction with mandatory native-cache reuse."""
import argparse
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "full-analysis-012"))
import full_audit as original


def aggregate_source(*args, **kwargs):
    source = original.aggregate_source(*args, **kwargs)
    return original.replace_once(source,
        "if sorted(str(p) for p in A.rglob('*') if p.is_file())!=[str(p) for p in archive_files]:",
        "if sorted(str(p) for p in A.rglob('*') if p.is_file())!=sorted(str(p) for p in archive_files):")


class CacheOnly(original.NativeCache):
    def run(self, core, rows, metadata, model, effort):
        claim = self.root / (metadata["thread_id"] + ".claim.json")
        if not claim.is_file():
            raise ValueError("supplement requires an already audited native thread")
        def forbidden(*args, **kwargs):
            raise AssertionError("a population supplement cannot execute the native audit core")
        return super().run(forbidden, rows, metadata, model, effort)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--batch", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    args = parser.parse_args()
    batch = args.batch.resolve()
    manifest = json.loads((batch / "PHASE-MANIFEST.json").read_bytes())
    original.require_terminal(batch, manifest)
    if args.run_id not in [x["run_id"] for x in manifest["schedule"]]:
        raise ValueError("run is outside the admitted population")
    output = batch / "postcapture" / args.run_id
    retained_path = output / "NATIVE-USAGE.json"
    retained_bytes = retained_path.read_bytes()
    retained = json.loads(retained_bytes)
    if "archive file population changed during audit" not in retained["errors"]:
        raise ValueError("no applicable population-check failure")
    for name, expected in retained["original_hashes_after"].items():
        if original.digest(Path(name).read_bytes()) != expected:
            raise ValueError("original audited input changed: " + name)
    result_path = output / "NATIVE-USAGE-POPULATION-SUPPLEMENT.json"
    command_path = output / "POPULATION-SUPPLEMENT-COMMAND.json"
    if result_path.exists() or command_path.exists():
        raise ValueError("supplement already claimed or published; preserve the first attempt")
    source = aggregate_source(batch, args.run_id, retained["expected_native_cwd"], output)
    original.publish(command_path, {"source": source,
        "retained_audit_sha256": original.digest(retained_bytes),
        "adapter_sha256": original.digest(Path(__file__).read_bytes()),
        "native_core_execution_allowed": False})
    stream = io.StringIO()
    with redirect_stdout(stream):
        exec(compile(source, str(command_path), "exec"),
             {"NativeCache":CacheOnly, "public_lifecycle":original.public_lifecycle})
    result = json.loads(stream.getvalue())
    for field in ("records", "native_usage", "category_totals", "turns", "threads"):
        if result[field] != retained[field]:
            raise ValueError("population correction changed accounting: " + field)
    if retained_path.read_bytes() != retained_bytes:
        raise ValueError("retained audit changed")
    result["supplement"] = {"reason": "consistent archive path ordering",
        "retained_audit": str(retained_path), "retained_audit_sha256": original.digest(retained_bytes),
        "native_core_execution_count": 0, "cached_thread_count": len(result["threads"])}
    original.publish(result_path, result)
    print(json.dumps({k:result[k] for k in ("run_id", "status", "errors", "native_usage", "supplement")}))


if __name__ == "__main__":
    main()
