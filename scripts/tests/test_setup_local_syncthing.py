"""Sanitized tests for local-only Syncthing bootstrap safeguards."""
import importlib.util
from pathlib import Path
import sys
import unittest
import xml.etree.ElementTree as ET
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
spec = importlib.util.spec_from_file_location('setup', Path(__file__).parents[1] / 'setup_local_syncthing.py')
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)
FIXTURE = b'''<configuration><device id="LOCAL"/><gui><apikey>fixture-only</apikey></gui><options><listenAddress>default</listenAddress><globalAnnounceEnabled>true</globalAnnounceEnabled><localAnnounceEnabled>true</localAnnounceEnabled><relaysEnabled>true</relaysEnabled><natEnabled>true</natEnabled><urAccepted>0</urAccepted></options><defaults><folder type="sendreceive"/></defaults></configuration>'''
class Configuration(unittest.TestCase):
    def test_data_roots_must_not_overlap(self):
        shared, boneyard = setup.validate_data_paths('/tmp/rawrarr/shared', '/tmp/rawrarr/boneyard')
        self.assertNotEqual(shared, boneyard)
        for left, right in (
            ('/tmp/rawrarr/shared', '/tmp/rawrarr/shared'),
            ('/tmp/rawrarr/shared', '/tmp/rawrarr/shared/boneyard'),
            ('/tmp/rawrarr/shared/boneyard', '/tmp/rawrarr/shared'),
        ):
            with self.subTest(left=left, right=right), self.assertRaises(ValueError):
                setup.validate_data_paths(left, right)
    def test_local_scope_and_idempotence(self):
        result = setup.configure(FIXTURE)
        root = ET.fromstring(result)
        folders = {folder.get('id'): folder for folder in root.findall('folder')}
        self.assertEqual(folders['rawrarr-shared'].get('path'), '/shared')
        self.assertEqual(folders['rawrarr-boneyard'].get('label'), 'Rawrarr boneyard')
        self.assertEqual(folders['rawrarr-boneyard'].get('path'), '/evt/boneyard')
        self.assertEqual(folders['rawrarr-boneyard'].find('device').get('id'), 'LOCAL')
        self.assertEqual(root.findtext('gui/apikey'), 'fixture-only')
        self.assertEqual(root.findtext('options/listenAddress'), 'tcp://0.0.0.0:22000')
        for key in ('globalAnnounceEnabled', 'localAnnounceEnabled', 'relaysEnabled', 'natEnabled'):
            self.assertEqual(root.findtext('options/' + key), 'true')
        self.assertEqual(setup.configure(result), result)
    def test_preserves_legacy_shared_folder_and_adds_boneyard(self):
        legacy = FIXTURE.replace(b'<defaults>', b'<folder id="rawrarr-shared" label="rawrarr shared application data" path="/shared" type="sendreceive"><device id="LOCAL"/></folder><defaults>')
        root = ET.fromstring(setup.configure(legacy))
        folders = {folder.get('id'): folder for folder in root.findall('folder')}
        self.assertEqual(folders['rawrarr-shared'].get('path'), '/shared')
        self.assertEqual(folders['rawrarr-boneyard'].get('path'), '/evt/boneyard')
    def test_reject_remote_peer(self):
        with self.assertRaises(ValueError):
            setup.configure(FIXTURE.replace(b'<gui>', b'<device id="REMOTE"/><gui>'))
    def test_reject_unrelated_folder(self):
        with self.assertRaises(ValueError):
            setup.configure(FIXTURE.replace(b'<gui>', b'<folder id="other" path="/elsewhere"/><gui>'))
    def test_reject_dtd_and_entity_declarations(self):
        for declaration in (
            b'<!DOCTYPE configuration [<!ENTITY x "expanded">]>',
            b'<!DOCTYPE configuration SYSTEM "file:///etc/passwd">',
        ):
            hostile = FIXTURE.replace(b'<configuration>', declaration + b'<configuration>')
            with self.subTest(declaration=declaration), self.assertRaisesRegex(ValueError, 'DTD and entity'):
                setup.configure(hostile)
if __name__ == '__main__':
    unittest.main()
