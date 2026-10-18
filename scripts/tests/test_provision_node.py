#!/usr/bin/env python3
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('provision_node', Path(__file__).parents[1] / 'provision_node.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class ProvisionTests(unittest.TestCase):
    def test_dry_run_and_secret_safe_apply(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'state'
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                module.provision(root, 'mini', 'storage', 'http://localhost:7297', 'localhost')
                self.assertFalse(root.exists())
                module.provision(root, 'mini', 'storage', 'http://localhost:7297', 'localhost', True)
            token_path = root / 'nodes/mini/token'
            token = token_path.read_text().strip()
            self.assertEqual(token_path.stat().st_mode & 0o777, 0o600)
            records = json.loads((root / 'coordinator/approved-nodes.json').read_text())
            self.assertEqual(records[0]['token_sha256'], hashlib.sha256(token.encode()).hexdigest())
            self.assertNotIn(token, output.getvalue())
            with self.assertRaises(ValueError):
                module.provision(root, 'mini', 'storage', 'http://localhost:7297', 'localhost', True)
            self.assertEqual(token, token_path.read_text().strip())

    def test_invalid_scope(self):
        with tempfile.TemporaryDirectory() as tmp:
            for node, role, origin, rp in [('../outside', 'storage', 'http://localhost', 'localhost'), ('mini', 'coordinator', 'http://localhost', 'localhost'), ('mini', 'storage', 'http://remote.test', 'remote.test'), ('mini', 'storage', 'https://remote.test', 'other.test')]:
                with self.assertRaises(ValueError):
                    module.provision(tmp, node, role, origin, rp, True)

if __name__ == '__main__':
    unittest.main()
