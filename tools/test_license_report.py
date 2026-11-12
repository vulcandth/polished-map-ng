import copy
import unittest

from license_report import verify_coverage


class LicenseCoverageTests(unittest.TestCase):
    def setUp(self):
        self.package = {'name': 'example', 'version': '1.0.0'}
        self.report = {
            'crates': [{'package': self.package}],
            'licenses': [{'id': 'MIT', 'text': 'Original terms',
                          'used_by': [{'crate': self.package}]}],
        }
        self.required = {('example', '1.0.0')}

    def test_omitted_dependency_or_license_fails_closed(self):
        self.assertEqual(verify_coverage(self.report, self.required), self.required)
        with self.assertRaisesRegex(ValueError, 'missing from cargo-about'):
            verify_coverage(self.report, self.required | {('missing', '2.0.0')})
        report = copy.deepcopy(self.report)
        report['licenses'] = []
        with self.assertRaisesRegex(ValueError, 'without license text'):
            verify_coverage(report, self.required)

    def test_empty_terms_and_unrecognized_owner_fail_closed(self):
        report = copy.deepcopy(self.report)
        report['licenses'][0]['text'] = ' '
        with self.assertRaisesRegex(ValueError, 'Empty license text'):
            verify_coverage(report, self.required)
        report = copy.deepcopy(self.report)
        report['licenses'][0]['used_by'][0]['crate'] = {'name': 'example', 'version': '2.0.0'}
        with self.assertRaisesRegex(ValueError, 'Unknown license owner'):
            verify_coverage(report, self.required)


if __name__ == '__main__':
    unittest.main()
