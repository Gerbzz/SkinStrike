"""Approved 2026-10-25 branding contract. Requires Pillow; does not modify files."""
from pathlib import Path
from hashlib import sha256
import json
import unittest
from PIL import Image
ROOT = Path(__file__).resolve().parents[1]
APP = 'SkinStrike'
APPROVED = 'e1523b9145d862eeb35dd566afe6d697984cf404f30cfd5c8f7d3b1ecefddff4'
class ApprovedBranding(unittest.TestCase):
    def test_exact_approved_source_and_readme(self):
        p = ROOT / 'assets/branding' / (APP + '.png')
        self.assertEqual(sha256(p.read_bytes()).hexdigest(), APPROVED)
        with Image.open(p) as image:
            self.assertEqual(image.size, (512,512))
        self.assertIn('assets/branding/' + APP + '.png', (ROOT/'README.md').read_text(encoding="utf-8"))
        self.assertEqual(json.loads((p.parent/'provenance.json').read_text(encoding="utf-8"))['sha256'], APPROVED)
    def test_selected_foundation_pin_and_canonical_source(self):
        import subprocess
        contract = json.loads((ROOT/'foundation/candidate.json').read_text(encoding='utf-8'))
        self.assertEqual(contract['candidate_commit'], 'c7d7a1a633e32a5fa90d8cd2ec8deaa405ca5b1c')
        entry = subprocess.check_output(['git','ls-files','--stage','--','foundation/subcraft'],cwd=ROOT,text=True)
        self.assertEqual(entry.split()[:2], ['160000',contract['candidate_commit']])
        source = ROOT/'foundation/subcraft/assets/branding/SkinStrike.png'
        self.assertTrue(source.is_file(), 'initialize pinned foundation submodule')
        self.assertEqual(source.read_bytes(), (ROOT/'assets/branding/SkinStrike.png').read_bytes())

if __name__ == "__main__":
    unittest.main()
