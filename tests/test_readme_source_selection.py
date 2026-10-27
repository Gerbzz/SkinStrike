"""Keep the selected 3D candidate distinct from the preserved legacy editor."""
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class SourceSelectionTest(unittest.TestCase):
    def test_main_page_shows_selected_3d_not_historical_2d_gallery(self):
        readme = (ROOT / 'README.md').read_text()
        self.assertIn('superseded', readme.lower())
        self.assertIn('superseded', (ROOT / 'AGENTS.md').read_text().lower())
        self.assertIn('SkinStrike-3D.ps1', readme)
        self.assertIn('skinstrike-active-3d-demo-c7d7a1a6.png', readme)
        self.assertNotIn('skinstrike-creator-gallery-current.png', readme)
        self.assertNotIn('skinstrike-paint-editor-current.png', readme)
        self.assertNotIn('cargo run --locked --features native-ui --bin skinstrike-ui', readme)
        self.assertIn('docs/HISTORICAL_2D_EDITOR.md', readme)

    def test_historical_evidence_and_project_boundary_are_preserved(self):
        history = (ROOT / 'docs/HISTORICAL_2D_EDITOR.md').read_text()
        for name in ('skinstrike-creator-gallery-current.png', 'skinstrike-paint-editor-current.png'):
            self.assertIn('../assets/screenshots/' + name, history)
            self.assertTrue((ROOT / 'assets/screenshots' / name).is_file())
        candidate = json.loads((ROOT / 'foundation/candidate.json').read_text())
        self.assertEqual(candidate['candidate_commit'], 'c7d7a1a633e32a5fa90d8cd2ec8deaa405ca5b1c')
        self.assertIn('no automatic conversion', candidate['project_boundary'])
        self.assertIn('SkinStrike-3D.ps1', (ROOT / 'AGENTS.md').read_text())


if __name__ == '__main__':
    unittest.main()
