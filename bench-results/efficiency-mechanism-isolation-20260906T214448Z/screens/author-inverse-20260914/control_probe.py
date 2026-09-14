"""Provider-free cross-tool signal receipt through the frozen child supervisor."""
import argparse
import json
from pathlib import Path
import sys
from author_inverse import load_host

parser = argparse.ArgumentParser()
parser.add_argument("--arm", required=True)
parser.add_argument("--artifact-dir", required=True)
args = parser.parse_args()
root = Path(args.artifact_dir)
root.mkdir()
host = load_host("identity")
result = host.execute_child(
    [sys.executable, "-c", "import time; time.sleep(45)"], root, None,
    root / "stdout", root / "stderr", 55, {})
host.save_json(root / "result.json", result)
print(json.dumps(result), flush=True)
