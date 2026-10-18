"""Sanitized tests for local-only Syncthing bootstrap safeguards."""
import importlib.util
from pathlib import Path
import unittest
import xml.etree.ElementTree as ET
spec = importlib.util.spec_from_file_location('setup', Path(__file__).parents[1] / 'setup_local_syncthing.py')
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)
FIXTURE = b'''<configuration><device id="LOCAL"/><gui><apikey>fixture-only</apikey></gui><options><listenAddress>default</listenAddress><globalAnnounceEnabled>true</globalAnnounceEnabled><localAnnounceEnabled>true</localAnnounceEnabled><relaysEnabled>true</relaysEnabled><natEnabled>true</natEnabled><urAccepted>0</urAccepted></options><defaults><folder type="sendreceive"/></defaults></configuration>'''
class Configuration(unittest.TestCase):
    def test_local_scope_and_idempotence(self):
        result = setup.configure(FIXTURE)
        root = ET.fromstring(result)
        self.assertEqual(root.find('folder').get('path'), '/shared')
        self.assertEqual(root.find('folder/device').get('id'), 'LOCAL')
        self.assertEqual(root.findtext('gui/apikey'), 'fixture-only')
        self.assertEqual(root.findtext('options/listenAddress'), 'tcp://127.0.0.1:22000')
        self.assertEqual(root.findtext('options/relaysEnabled'), 'false')
        self.assertEqual(setup.configure(result), result)
    def test_reject_remote_peer(self):
        with self.assertRaises(ValueError):
            setup.configure(FIXTURE.replace(b'<gui>', b'<device id="REMOTE"/><gui>'))
    def test_reject_unrelated_folder(self):
        with self.assertRaises(ValueError):
            setup.configure(FIXTURE.replace(b'<gui>', b'<folder id="other" path="/elsewhere"/><gui>'))
if __name__ == '__main__':
    unittest.main()
