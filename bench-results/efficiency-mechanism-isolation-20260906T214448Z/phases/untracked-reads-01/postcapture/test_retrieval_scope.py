import pathlib
import types
import unittest

path=pathlib.Path(__file__).resolve().with_name('replay_retrieval_only.py')
M=types.ModuleType('retrieval_test');M.__file__=str(path)
exec(compile(path.read_bytes(),str(path),'exec'),M.__dict__)


class ScopeTests(unittest.TestCase):
    def test_other_bundle_witness_does_not_prove_this_read(self):
        read={'eligible':True,'selected_candidate':'baseline','bundle':{'path':'/B'}}
        call={'classification':'selected_content_delivered','witnesses':[{'bundle_path':'/A'}]}
        self.assertEqual(M.read_disposition(read,[call]),'unresolved_or_no_content')
        call['witnesses'].append({'bundle_path':'/B'})
        self.assertEqual(M.read_disposition(read,[call]),'selected_content_native_output')

    def test_compound_error_does_not_erase_prior_content(self):
        output='some source\nrg: unrecognized flag bad\n'
        self.assertEqual(M.failure_kind("sed -n '1,2p' /A && rg --bad /B",output,2),'failure_with_unresolved_prior_content')
        self.assertEqual(M.failure_kind("rg -n '--bad' /B",'rg: unrecognized flag --bad\n',2),'failed_before_content_delivery')

    def test_shared_line_is_not_cross_bundle_origin(self):
        self.assertEqual(M.line_values('/A:17:identical public source line','/B',True),[])
        self.assertEqual(M.line_values('identical public source line','/B',True),[])
        self.assertEqual(M.line_values('/B:17:identical public source line','/B',True),[('identical public source line',17)])


if __name__=='__main__':unittest.main()
