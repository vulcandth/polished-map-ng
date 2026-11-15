"""Guard the restored upstream manual against another wholesale rewrite."""
import unittest

from help_assets import ASSETS, Document, expand


class HelpAdaptationTests(unittest.TestCase):
    def test_original_prose_structure_and_attributed_controls_are_retained(self):
        original = Document()
        original.feed(expand((ASSETS / 'master-help.html.inc').read_text(encoding='utf-8'), False))
        current = Document()
        current.feed((ASSETS / 'ng-help.html').read_text(encoding='utf-8'))
        # Most original blocks should survive verbatim. This permits targeted
        # accuracy edits but rejects the previous newly authored replacement.
        self.assertGreaterEqual(sum(block in current.blocks for block in original.blocks),
                                len(original.blocks) * 2 // 3)
        self.assertEqual(sum(b['kind'] == 'rule' for b in original.blocks),
                         sum(b['kind'] == 'rule' for b in current.blocks))
        prose = [''.join(s['text'] for s in b['spans']) for b in current.blocks]
        anchors = ['The sidebar array uses the mouse:',
                   'The block editor window has some simple mouse controls:',
                   'And some more complex ones for placing multiple tiles at once, with the selected tile as the top-left:']
        positions = [prose.index(anchor) for anchor in anchors]
        self.assertEqual(positions, sorted(positions))
        joined = '\n'.join(prose)
        for phrase in ['Monochrome:', '256 Tiles:', '512 Tiles:', 'X flip:', 'Y flip:',
                       'Priority:', 'Color:', 'Ctrl+Shift+E', 'Polished Map NG']:
            self.assertIn(phrase, joined)
        for stale in ['implemented with C++', 'FLTK', 'install.bat', 'Ctrl+Shift+T',
                      'Polished Map++', 'unified development is in progress']:
            self.assertNotIn(stale, joined)


if __name__ == '__main__':
    unittest.main()
