"""Regression for a stale initial count inside the actual-publication test."""
import json
from pathlib import Path
import unittest
from unittest.mock import patch

import progress_counter as core
import test_progress_end_to_end as original


class PublicationAdvanceTests(unittest.TestCase):
    def test_existing_actual_publication_check_accepts_a_valid_completion(self):
        fixture = original.CompletePlanTests()
        fixture.setUp()
        fixture.close("P01")
        hypothesis, note = fixture.visible()
        replacements = {
            core.STUDY / "PROGRESS-CHECKLIST.json": json.dumps(fixture.ledger),
            core.STUDY / "PROGRESS-PUBLICATION-HISTORY.json": json.dumps(fixture.pubs()),
            core.ROOT / "hypotesis.md": hypothesis,
            core.ROOT / "ephemeral-note.md": note,
        }
        read_text = Path.read_text

        def read(path, *args, **kwargs):
            if path in replacements:
                return replacements[path]
            return read_text(path, *args, **kwargs)

        # Fixture completion evidence is declarative. Production evidence validation
        # remains unchanged and is exercised against the actual repository separately.
        with patch.object(Path, "read_text", read), patch.object(core, "validate_evidence_files") as evidence:
            original.CompletePlanTests().test_actual_publication()
        evidence.assert_called_once_with(fixture.ledger)


if __name__ == "__main__":
    unittest.main()
