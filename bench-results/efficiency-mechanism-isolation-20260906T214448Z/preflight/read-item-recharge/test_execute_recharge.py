import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
from types import SimpleNamespace

P=Path(__file__).with_name('execute_recharge.py')
spec=importlib.util.spec_from_file_location('execute_recharge',P)
d=importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)


class DriverTests(unittest.TestCase):
    def test_missing_writer_precedes_any_execution(self):
        with tempfile.TemporaryDirectory() as tmp:
            scope=Path(tmp)/'scope.json';output=Path(tmp)/'result.json'
            scope.write_bytes(d.canonical({'rows':[{'run_id':'r','output':str(output),'attempt':str(Path(tmp)/'attempt.json')}]}))
            args=SimpleNamespace(scope=scope,scope_sha256=d.sha(scope.read_bytes()),run_id='r')
            with mock.patch.object(d.argparse.ArgumentParser,'parse_args',return_value=args),mock.patch.object(d.shutil,'which',return_value=None),mock.patch.object(d,'execute',return_value=(str(output),{})) as execute:
                with self.assertRaises(ValueError):d.main()
                execute.assert_not_called()

    def test_retained_attempt_prevents_automatic_second_execution(self):
        with tempfile.TemporaryDirectory() as tmp:
            scope=Path(tmp)/'scope.json';attempt=Path(tmp)/'attempt.json';attempt.write_text('{}')
            scope.write_bytes(d.canonical({'rows':[{'run_id':'r','output':str(Path(tmp)/'result.json'),'attempt':str(attempt)}]}))
            args=SimpleNamespace(scope=scope,scope_sha256=d.sha(scope.read_bytes()),run_id='r')
            with mock.patch.object(d.argparse.ArgumentParser,'parse_args',return_value=args),mock.patch.object(d.shutil,'which',return_value='/writer'),mock.patch.object(d,'execute') as execute:
                with self.assertRaises(ValueError):d.main()
                execute.assert_not_called()

    def test_failed_result_publication_preserves_marker_and_blocks_retry(self):
        with tempfile.TemporaryDirectory() as tmp:
            scope=Path(tmp)/'scope.json';attempt=Path(tmp)/'attempt.json';output=Path(tmp)/'result.json'
            scope.write_bytes(d.canonical({'rows':[{'run_id':'r','output':str(output),'attempt':str(attempt)}]}))
            args=SimpleNamespace(scope=scope,scope_sha256=d.sha(scope.read_bytes()),run_id='r')
            def publish(writer,path,result):
                if Path(path)==attempt:attempt.write_bytes(d.canonical(result));return 'hash'
                raise ValueError('retained synthetic publication failure')
            with mock.patch.object(d.argparse.ArgumentParser,'parse_args',return_value=args),mock.patch.object(d.shutil,'which',return_value='/writer'),mock.patch.object(d,'publish',side_effect=publish),mock.patch.object(d,'execute',return_value=(str(output),{})) as execute:
                with self.assertRaises(ValueError):d.main()
                self.assertTrue(attempt.exists());self.assertFalse(output.exists())
                with self.assertRaises(ValueError):d.main()
                self.assertEqual(execute.call_count,1)

    def test_duplicate_keys_and_incomplete_closed_tail_rejected(self):
        with self.assertRaises(ValueError):d.decode(b'{"a":1,"a":2}')
        with self.assertRaises(ValueError):list(d.jsonl(b'{"type":"event"}'))
        self.assertEqual(len(list(d.jsonl(b'{"type":"event"}\n'))),1)

    def test_partial_raw_metadata_keeps_frozen_unknown_path(self):
        rows=[{'params':{'usageMetadata':None}}, {'params':{'usageMetadata':{'metadata':{'attribution':None}}}}]
        self.assertEqual(d.metadata_rows(rows),2)

    def test_all_accepted_turns_including_usageless_failed_are_owned(self):
        delivery={'accepted_inputs':[{'thread_id':'thread','turn_id':'done'},
                                    {'thread_id':'thread','turn_id':'failed-without-usage'}],
                  'native_contexts':[{'thread_id':'thread','source':'/native'}]}
        self.assertEqual(d.owned_turns(delivery),{'thread':{'done','failed-without-usage'}})
        delivery['accepted_inputs'][1]['turn_id']=True
        with self.assertRaises(ValueError):d.owned_turns(delivery)

    def test_pinned_reader_rejects_unlisted_changed_and_symlink_sources(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'source';path.write_bytes(b'bytes')
            reader=d.Reader({str(path):hashlib.sha256(b'bytes').hexdigest()})
            self.assertEqual(reader.read(str(path)),b'bytes')
            with self.assertRaises(ValueError):reader.read(str(Path(tmp)/'missing'))
            path.write_bytes(b'changed')
            with self.assertRaises(ValueError):reader.read(str(path))
            link=Path(tmp)/'link';link.symlink_to(path)
            with self.assertRaises(ValueError):d.Reader({str(link):hashlib.sha256(b'changed').hexdigest()}).read(str(link))

    def test_saved_ledger_hash_and_all_response_ownership_required(self):
        ledger={'resp':{'thread_id':'thread','turn_id':'no-usage-turn','usage':{
            'input_tokens':7,'cached_input_tokens':2,'output_tokens':3,'reasoning_output_tokens':1}}}
        expected={'count':1,'canonical_sha256':d.sha(d.canonical(ledger))}
        self.assertEqual(d.ledger(ledger,expected,{'thread':{'no-usage-turn'}}),ledger)
        with self.assertRaises(ValueError):d.ledger(ledger,expected,{'thread':{'different'}})
        ledger['resp']['usage']['input_tokens']=8
        with self.assertRaises(ValueError):d.ledger(ledger,expected,{'thread':{'no-usage-turn'}})

    def test_whole_response_order_has_one_capture_per_thread(self):
        rows=[{'method':'rawResponse/completed','params':{'threadId':'thread','responseId':'r1'},'_audit_source':'/a'},
              {'method':'rawResponse/completed','params':{'threadId':'thread','responseId':'r2'},'_audit_source':'/b'}]
        with self.assertRaises(ValueError):d.one_capture_per_thread(rows)

    def test_native_counter_identity_replay_is_exact_not_retoting(self):
        usage={'input_tokens':7,'cached_input_tokens':2,'output_tokens':3,'reasoning_output_tokens':1}
        ledger={'resp':{'thread_id':'thread','turn_id':'turn','usage':usage,
                        'sources':[{'path':'/native','native_line':4}]}}
        native={'/native':{4:{'type':'token_usage_record','payload':{
            'response_id':'resp','thread_id':'thread','turn_id':'turn','usage':dict(usage)}}}}
        self.assertEqual(d.native_counter_replay(ledger,native),1)
        native['/native'][4]['payload']['usage']['input_tokens']=8
        with self.assertRaises(ValueError):d.native_counter_replay(ledger,native)


if __name__=='__main__':unittest.main()
