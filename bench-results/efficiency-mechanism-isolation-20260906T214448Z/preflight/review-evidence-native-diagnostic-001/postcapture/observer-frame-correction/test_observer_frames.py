import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('observer_frames', HERE / 'audit_observer_frames.py')
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


def fixture():
    before = [dict(id='init', method='initialize', params=dict(capabilities=dict(experimentalApi=True))),
              dict(id=2, method='thread/start', params=dict(cwd='/project')),
              dict(id='turn', method='turn/start', params=dict(threadId='thread', input=[dict(type='text', text='unchanged € request')]))]
    after = copy.deepcopy(before)
    after[0]['params']['capabilities']['optOutNotificationMethods'] = ['rawResponseItem/completed']
    after[1]['params']['experimentalRawEvents'] = True
    return before, after


def encode(rows):
    return [json.dumps(row, ensure_ascii=False, separators=(',', ':')).encode() + b'\n' for row in rows]


def inputs(before, after):
    a, b = encode(before), encode(after)
    decisions = []
    for i, (x, y) in enumerate(zip(a, b)):
        value = before[i]
        if value['method'] not in ('initialize', 'thread/start'):
            continue
        decisions.append(dict(method=value['method'], id=value['id'], changed=before[i] != after[i],
                              original_sha256=hashlib.sha256(x).hexdigest(), forwarded_sha256=hashlib.sha256(y).hexdigest(),
                              original_bytes=len(x), forwarded_bytes=len(y), observed_monotonic_ns=i))
    return b''.join(a), b''.join(b), decisions


def closed_input():
    root = audit.DIAGNOSTIC
    def ref(path): return dict(path=str(path), sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    source_input = root/'postcapture/REVIEW-EVIDENCE-SOURCE-INPUT.json'
    old = json.loads(source_input.read_text())
    return dict(schema='work-leaf-review-observer-frame-input-v1',
                helper_sha256=hashlib.sha256(Path(audit.__file__).read_bytes()).hexdigest(),
                source_input=ref(source_input), observer_config=ref(root/'observation/observer-config.json'),
                captures=[dict(path=c['path'], settings=ref(Path(c['path'])/'raw-response-usage.json'),
                               journal=ref(Path(c['path'])/'raw-response-rewrites.jsonl')) for c in old['captures']])


class FrameTests(unittest.TestCase):
    def prove(self, before, after, decisions=None):
        original, forwarded, journal = inputs(before, after)
        return audit.prove_frames(original, forwarded, journal if decisions is None else decisions, copy.deepcopy(audit.SETTINGS))

    def test_known_additions_and_unchanged_turn(self):
        before, after = fixture()
        proof = self.prove(before, after)
        self.assertEqual(proof['changed_frame_lines'], [1, 2])
        self.assertEqual(proof['frame_count'], 3)
        self.assertTrue(audit.matches_proof(proof, before, after))
        original = audit.original_modules()[1]
        self.assertFalse(original.typed_equal(before, after))

    def test_already_correct_metadata_stays_identity(self):
        _, after = fixture()
        self.assertEqual(self.prove(after, copy.deepcopy(after))['changed_frame_lines'], [])

    def test_prompt_id_order_and_extra_capability_are_rejected(self):
        for case in ('prompt', 'id', 'order', 'capability', 'notification', 'experimental-api'):
            with self.subTest(case=case):
                a, b = fixture()
                if case == 'prompt': b[2]['params']['input'][0]['text'] = 'changed'
                elif case == 'id': b[1]['id'] = '2'
                elif case == 'order': b[1], b[2] = b[2], b[1]
                elif case == 'capability': b[0]['params']['capabilities']['other'] = True
                elif case == 'notification': b[0]['params']['capabilities']['optOutNotificationMethods'].append('other')
                else: a[0]['params']['capabilities']['experimentalApi'] = False
                with self.assertRaises(ValueError): self.prove(a, b)

    def test_boolean_rpc_id_is_not_integer(self):
        a, b = fixture(); a[1]['id'] = b[1]['id'] = True
        with self.assertRaises(ValueError): self.prove(a, b)

    def test_journal_is_complete_exact_and_typed(self):
        for case in ('missing', 'extra', 'hash', 'bytes', 'id', 'changed'):
            with self.subTest(case=case):
                a, b = fixture(); _, _, journal = inputs(a, b)
                if case == 'missing': journal.pop()
                elif case == 'extra': journal.append(copy.deepcopy(journal[-1]))
                elif case == 'hash': journal[0]['original_sha256'] = '0' * 64
                elif case == 'bytes': journal[0]['original_bytes'] += 1
                elif case == 'id': journal[1]['id'] = True
                else: journal[0]['changed'] = 1
                with self.assertRaises(ValueError): self.prove(a, b, journal)

    def test_settings_and_unbound_proof_reject(self):
        a, b = fixture(); original, forwarded, journal = inputs(a, b)
        settings = copy.deepcopy(audit.SETTINGS); settings['enabled'] = False
        with self.assertRaises(ValueError): audit.prove_frames(original, forwarded, journal, settings)
        proof = self.prove(a, b); a[2]['params']['input'][0]['text'] = 'changed'
        self.assertFalse(audit.matches_proof(proof, a, b))

    def test_unchanged_frame_bytes_cannot_be_reserialized(self):
        a, b = fixture(); original, forwarded, journal = inputs(a, b)
        forwarded = forwarded.replace('unchanged € request'.encode(), 'unchanged \\u20ac request'.encode())
        with self.assertRaises(ValueError): audit.prove_frames(original, forwarded, journal, audit.SETTINGS)

    def test_actual_closed_capture_full_source_join_retains_original_failure(self):
        original = audit.DIAGNOSTIC/'postcapture/REVIEW-EVIDENCE-SOURCE-AUDIT.json'
        before = original.read_bytes()
        result = audit.audit_sources(closed_input(), audit.original_modules()[0].Sources())
        self.assertEqual(result['errors'], [])
        self.assertEqual(result['status'], 'available')
        self.assertEqual(result['terminal']['exit_code'], 101)
        self.assertEqual(result['reviews'][0]['retrieval']['status'], 'complete')
        self.assertEqual(original.read_bytes(), before)
        self.assertEqual(json.loads(before)['errors'], ['original/forwarded frames differ'])

    def test_unenabled_route_cannot_borrow_proof_and_retains_verified_terminal(self):
        Base = audit.original_modules()[0].Sources
        class DisabledRoute(Base):
            def json(self, ref):
                value = super().json(ref)
                if ref['path'].endswith('/start.json'):
                    value['raw_response_usage'] = False
                return value
        result = audit.audit_sources(closed_input(), DisabledRoute())
        self.assertEqual(result['status'], 'unverifiable')
        self.assertTrue(any('route is not enabled' in x for x in result['errors']))
        self.assertEqual(result['terminal']['exit_code'], 101)


if __name__ == '__main__':
    unittest.main()
