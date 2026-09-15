"""One bounded real host/fix/direct input-route qualification, not a feature run."""
import argparse
import json
import os
from pathlib import Path
import sys
import time

import full_workflow as f


def host_command(repo, artifacts, stage, prompt, thread, deadline):
    command = [sys.executable, str(f.HERE/'host_custody.py'), '--repo', str(repo),
               '--artifact-dir', str(artifacts), '--feature', 'diagnostic', '--stage', stage,
               '--prompt-file', str(prompt), '--codex-bin', str(f.HERE/'input_provider'),
               '--deadline', str(deadline), '--serialized-feedback', '--model', 'gpt-5.5']
    if thread:
        command += ['--resume-thread', thread]
    return command


def main():
    parser = argparse.ArgumentParser()
    for key in ('arm','artifact-dir','repo'):
        parser.add_argument('--'+key, required=True)
    args = parser.parse_args()
    f.require(args.arm == 'P09' and os.environ.get('WORK_LEAF_BENCH_FULL_INVERSE') == '1',
              'explicit verification identity required')
    root, repo = Path(args.artifact_dir), Path(args.repo)
    host = f.load_host()
    deadline = time.monotonic()+120
    thread = None
    stages = json.loads((root/'VERIFY-STAGES.json').read_text())
    for spec in stages:
        stage = spec['stage']
        env = dict(os.environ)
        env['WORK_LEAF_OBSERVER_ROLE'] = stage
        output = root/stage
        output.mkdir()
        prompt = root/(stage+'.prompt.txt')
        if spec['kind'] == 'host':
            command = host_command(repo, output/'host', stage, prompt, thread, deadline)
        else:
            f.require(thread is not None, 'real review-resume requires the original thread')
            command = [str(f.HERE/'input_provider'),'--cd',str(repo),'--sandbox','read-only',
                       '--ask-for-approval','never','--model','gpt-5.5','exec','--color','never',
                       'resume','--json','-o',str(output/'final.txt'),thread,'-']
        host.save_json(output/'request.json', {'argv':command,'expected_thread':thread})
        status = host.execute_child(command, repo, prompt, output/'stdout.jsonl',
                                    output/'stderr.txt', max(0,deadline-time.monotonic()), env)
        host.save_json(output/'exit.json',status)
        f.require(status['exit_code']==0 and not status['timed_out'] and not status['cancelled_signal'],
                  'real route invocation failed; no retry')
        if spec['kind']=='host':
            result=json.loads((output/'host/result.json').read_text())
            f.require(result['completed'] and result['clean'] and not result['accepted_commits'],
                      'host qualification changed files or did not finish')
            f.require(thread is None or result['thread_id']==thread, 'host resume thread changed')
            thread=result['thread_id']
        else:
            observed, final, _ = host.owned_final(output/'stdout.jsonl',output/'final.txt',thread)
            f.require(observed==thread and final.strip()=='WORK_LEAF_REAL_FULL_ROUTE_OK',
                      'direct resume marker or ownership differs')
    host.save_json(root/'DIAGNOSTIC-RESULT.json',{'completed':True,'thread_id':thread,
                   'scope':'actual host result, same-author stage resume and direct resume route only'})
    return 0


if __name__=='__main__':
    raise SystemExit(main())
