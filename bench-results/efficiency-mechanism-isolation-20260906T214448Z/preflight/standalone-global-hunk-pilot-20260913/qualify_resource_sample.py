import contextlib, io, json, tempfile
from pathlib import Path
code="import json\nfrom pathlib import Path\nfrom datetime import datetime, timezone\nP=Path(\"/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/standalone-global-hunk-pilot-01\")\nm=json.loads((P/\"PHASE-MANIFEST.json\").read_text())\nrows=[]\nfor r in m[\"schedule\"]:\n    live=list(Path(r[\"runtime_dir\"]).glob(\"work-leaf-3feature-sequential-bench.*/runs\"))\n    if len(live)>1: raise ValueError(\"multiple live populations\")\n    root=live[0] if live else Path(r[\"artifact\"])/\"runs\"\n    files=sorted(set(root.glob(\"*.jsonl\")) | set(root.glob(\"*.host/invocation-*/stdout.jsonl\")))\n    raw=0; count=0; details=[]; partial=0\n    for p in files:\n        subtotal=0; n=0\n        data=p.read_bytes()\n        for i,line in enumerate(data.splitlines(keepends=True)):\n            try: event=json.loads(line)\n            except json.JSONDecodeError:\n                if not line.endswith(b\"\\n\"): partial+=1; continue\n                raise\n            if event.get(\"type\")!=\"turn.completed\": continue\n            u=event[\"usage\"]\n            value=u[\"input_tokens\"]+u[\"output_tokens\"]\n            if type(value) is not int or value<0: raise ValueError(\"invalid usage\")\n            subtotal+=value; n+=1\n        raw+=subtotal; count+=n\n        if n: details.append({\"file\":str(p.relative_to(root)),\"terminals\":n,\"reported_raw\":subtotal})\n    terminal=P/\"logs\"/(r[\"run_id\"]+\".exit.json\")\n    closure=json.loads(terminal.read_text()) if terminal.exists() else None\n    rows.append({\"id\":r[\"run_id\"],\"population\":str(root),\"files\":len(files),\"completed_public_turns\":count,\"reported_raw\":raw,\"partial_lines\":partial,\"terminal\":closure,\"details\":details})\nprint(json.dumps({\"at_utc\":datetime.now(timezone.utc).isoformat(),\"source\":\"completed-public resource-only; not reconciled usage\",\"threshold_per_run\":25000000,\"stop_wave\":any(r[\"reported_raw\"]>=25000000 for r in rows),\"phase_terminal\":(P/\"PHASE-RESULT.json\").exists(),\"runs\":rows},indent=2))\n"
with tempfile.TemporaryDirectory(prefix="wl-resource-qualification-") as tmp:
    phase=Path(tmp)
    runtime=phase/"owned"; artifact=phase/"artifact"
    row={"run_id":"qualification-only","runtime_dir":str(runtime),"artifact":str(artifact)}
    (phase/"PHASE-MANIFEST.json").write_text(json.dumps({"schedule":[row]}))
    (phase/"logs").mkdir()
    body=code.replace("P=Path(\"/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/standalone-global-hunk-pilot-01\")", "P=Path("+repr(str(phase))+")")
    assert body != code
    def sample():
        output=io.StringIO()
        with contextlib.redirect_stdout(output): exec(compile(body,"retained-resource-command","exec"),{})
        return json.loads(output.getvalue())["runs"][0]
    empty=sample(); assert empty["files"]==0 and empty["completed_public_turns"]==0
    live=runtime/"work-leaf-3feature-sequential-bench.fixture"/"runs"
    host=live/"feature.host"/"invocation-0001"; host.mkdir(parents=True)
    terminal=lambda i,o:json.dumps({"type":"turn.completed","usage":{"input_tokens":i,"output_tokens":o,"cached_input_tokens":i}})
    (live/"review.jsonl").write_text(terminal(100,20)+"\n")
    (host/"stdout.jsonl").write_text(terminal(30,4)+"\n"+'{"type":')
    (host/"events.jsonl").write_text(terminal(999,999)+"\n")
    (artifact/"runs").mkdir(parents=True)
    (artifact/"runs"/"duplicate.jsonl").write_text(terminal(888,888)+"\n")
    got=sample(); assert (got["files"],got["completed_public_turns"],got["reported_raw"],got["partial_lines"])==(2,2,154,1),got
    (live/"review.jsonl").write_text(terminal(100,20)+"\n"+"invalid full line\n")
    try: sample()
    except json.JSONDecodeError: pass
    else: raise AssertionError("malformed completed line must fail")
    (live/"review.jsonl").write_text(terminal(100,20)+"\n")
    second=runtime/"work-leaf-3feature-sequential-bench.second"/"runs"; second.mkdir(parents=True)
    try: sample()
    except ValueError as e: assert "multiple live" in str(e)
    else: raise AssertionError("ambiguous live ownership accepted")
print(json.dumps({"source":"retained V4 resource command; sole production delta phase path","checks":["empty means no observed completed usage, not zero consumption","ordinary plus host union","no archive/sidecar double count","cached input not added twice","partial tail retained as partial","malformed complete line rejected","ambiguous runtime rejected"],"expected_observed_raw":154,"pass":True}))
