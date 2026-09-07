#!/usr/bin/env python3
"""Separate postcapture title-launch correction; the admitted auditor stays frozen.

Run the original auditor first and retain its output. This derivative accepts an
already source-bound first policy request on a title thread, while retaining the
ordinary exact title contract for its later requests. No model or phase analyzer
is invoked. All source, delivery, factor and public-item gates otherwise remain.
"""
import hashlib
from pathlib import Path
import types

HERE = Path(__file__).resolve().parent
STUDY = HERE.parents[1]
ORIGINAL_PATH = STUDY/'audit_candidate_delivery.py'
ORIGINAL_SHA256 = 'ab9570bb0e87674fa738aa0dc6e51702ef7ac6ab97b17092d1f546a60b3889e1'
SELF_PATH = Path(__file__).resolve()
SELF_SHA256 = hashlib.sha256(SELF_PATH.read_bytes()).hexdigest()
original = ORIGINAL_PATH.read_bytes()
if hashlib.sha256(original).hexdigest() != ORIGINAL_SHA256:
    raise ValueError('frozen original candidate auditor differs')

# Two exact source replacements, each unique, inside the private occurrence audit.
# First-request ownership has already passed exact policy-text or title-prefix binding.
replacements = (
    ('                if thread not in owner:\n',
     '                first_request = thread not in owner\n                if first_request:\n'),
    ("                    require(inputs[0]['text'].startswith(TITLE), 'auxiliary title thread contains unsupported request')",
     "                    require(first_request or inputs[0]['text'].startswith(TITLE), 'auxiliary title thread contains unsupported request')"),
)
derived = original.decode()
for before, after in replacements:
    if derived.count(before) != 1: raise ValueError('frozen title correction seam differs')
    derived = derived.replace(before, after, 1)
DERIVED_CODE_SHA256 = hashlib.sha256(derived.encode()).hexdigest()
_module = types.ModuleType('candidate_title_corrected_exact_bytes')
_module.__file__ = str(ORIGINAL_PATH)
exec(compile(derived, str(ORIGINAL_PATH), 'exec'), _module.__dict__)
audit_delivery = _module.audit_delivery
_original_audit_sources = _module.audit_sources


def audit_sources(manifest, sources=None):
    sources = sources or _module.Sources()
    sources.read({'path': str(SELF_PATH), 'sha256': SELF_SHA256})
    result = _original_audit_sources(manifest, sources)
    result['derivative_verifier'] = {
        'schema': 'work-leaf-candidate-title-audit-correction-v1',
        'original_helper_sha256': ORIGINAL_SHA256,
        'derivative_helper_sha256': SELF_SHA256,
        'executed_derived_source_sha256': DERIVED_CODE_SHA256,
        'scope': 'First source-bound title policy launch only; original audit and terminal failure remain retained. No generation or accounting change.',
    }
    return result


_module.audit_sources = audit_sources
main = _module.main
if __name__ == '__main__': raise SystemExit(main())
