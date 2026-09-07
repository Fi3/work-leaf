"""Provider-free regression for the ordinary policy-wrapped title launch."""
import copy
import hashlib
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

STUDY = Path(__file__).resolve().parents[2]
ORIGINAL_SHA = 'ab9570bb0e87674fa738aa0dc6e51702ef7ac6ab97b17092d1f546a60b3889e1'
FIXTURE_SHA = 'c5ae757165a23603fe1186cc76365444cbbce042e387b93f9b812e20ce348454'


def load_exact(name, path, expected):
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == expected
    module = types.ModuleType(name); module.__file__ = str(path)
    exec(compile(data, str(path), 'exec'), module.__dict__)
    return module


ORIGINAL = load_exact('frozen_title_original', STUDY/'audit_candidate_delivery.py', ORIGINAL_SHA)
sys.modules['audit_candidate_delivery'] = ORIGINAL
FIXTURES = load_exact('frozen_title_fixtures', STUDY/'test_audit_candidate_delivery.py', FIXTURE_SHA)
DERIVATIVE_PATH = Path(__file__).with_name('audit_title_delivery.py')
SUBJECT = load_exact('title_derivative', DERIVATIVE_PATH, hashlib.sha256(DERIVATIVE_PATH.read_bytes()).hexdigest())


def full_title_fixture():
    row = FIXTURES.policy('title-agent')
    request = ORIGINAL.TITLE+'A real first feature request'
    for key in ('original_prompt', 'candidate_prompt'):
        row[key] = row[key].replace('Literal request λ', request)
    row.update(original_bytes=len(row['original_prompt'].encode()), candidate_bytes=len(row['candidate_prompt'].encode()), selected_bytes=len(row['candidate_prompt'].encode()))
    trace, captures = FIXTURES.fixture([FIXTURES.policy(), FIXTURES.read(), row])
    for number in (100, 101): FIXTURES.extra_turn(captures[0], ORIGINAL.TITLE+f'Feature {number}', 'thread-title-agent', number)
    return trace, captures


class TitleDeliveryTests(unittest.TestCase):
    def test_actual_policy_wrapped_title_launch_and_two_plain_followups(self):
        result = SUBJECT.audit_delivery(*full_title_fixture(), FIXTURES.RUN, FIXTURES.CONDITION)
        self.assertEqual(result['status'], 'available', result['errors'])
        self.assertEqual(result['auxiliary_title_turns'], 3)
        self.assertEqual(result['delivered']['candidate_policy'], 2)

    def test_old_failure_is_preserved_explicitly(self):
        result = ORIGINAL.audit_delivery(*full_title_fixture(), FIXTURES.RUN, FIXTURES.CONDITION)
        self.assertEqual(result['status'], 'unverifiable')
        self.assertIn('auxiliary title thread contains unsupported request', result['errors'])

    def test_other_title_requests_and_unknown_owner_remain_rejected(self):
        for mutation in ('followup', 'untraced-policy', 'wrong-role'):
            trace, captures = full_title_fixture()
            if mutation == 'followup':
                captures[0]['clients'][-1]['params']['input'][0]['text'] = 'Unexpected non-title task'
                captures[0]['forwarded'][-1] = copy.deepcopy(captures[0]['clients'][-1])
            elif mutation == 'untraced-policy': trace.pop()
            else: trace[-1]['agent_id'] = 'linearize'
            result = SUBJECT.audit_delivery(trace, captures, FIXTURES.RUN, FIXTURES.CONDITION)
            self.assertEqual(result['status'], 'unverifiable', mutation)

    def test_no_title_path_is_unchanged(self):
        fixture = FIXTURES.fixture()
        self.assertEqual(SUBJECT.audit_delivery(*fixture, FIXTURES.RUN, FIXTURES.CONDITION),
                         ORIGINAL.audit_delivery(*fixture, FIXTURES.RUN, FIXTURES.CONDITION))

    def test_complete_source_wrapper_retains_old_failure_and_terminal_outcome(self):
        fixture = full_title_fixture()
        with tempfile.TemporaryDirectory() as tmp, patch.object(FIXTURES, 'fixture', return_value=fixture):
            manifest = FIXTURES.SourceTests().manifest(Path(tmp))
            old = ORIGINAL.audit_sources(manifest)
            result = SUBJECT.audit_sources(manifest)
            self.assertEqual(old['status'], 'unverifiable')
            self.assertIn('auxiliary title thread contains unsupported request', old['errors'])
            self.assertEqual(result['status'], 'available', result['errors'])
            self.assertEqual(result['auxiliary_title_turns'], 3)
            self.assertEqual(result['terminal'], old['terminal'])
            self.assertEqual(result['terminal']['exit_code'], 1)
            self.assertEqual(result['source_sha256'][str(DERIVATIVE_PATH)], SUBJECT.SELF_SHA256)
            self.assertEqual(result['derivative_verifier']['original_helper_sha256'], ORIGINAL_SHA)
            self.assertEqual(result['derivative_verifier']['executed_derived_source_sha256'], SUBJECT.DERIVED_CODE_SHA256)
            self.assertEqual({key: value for key, value in result['source_sha256'].items() if key != str(DERIVATIVE_PATH)}, old['source_sha256'])
            output = Path(tmp)/'derived.json'
            SUBJECT._module.write_new(output, result)
            with self.assertRaises(FileExistsError): SUBJECT._module.write_new(output, result)


if __name__ == '__main__': unittest.main()
