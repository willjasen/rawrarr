"""Check soak reports distinguish measured success from rollout qualification."""

import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import rayfish_docker


class SoakTests(unittest.TestCase):
    def peers(self):
        peers = [Mock(), Mock()]
        for index, peer in enumerate(peers):
            peer.name = f"synthetic-peer-{index}"
            peer.status.return_value = {"endpoint": str(index)}
            peer.request.side_effect = [[200, "hash", 10], [200, "hash", 10], [401, "unauthorized", 0]]
        return peers

    def test_success_records_samples_bytes_and_separate_discovery_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            with patch.object(rayfish_docker.time, "monotonic", side_effect=[100, 101, 102, 103]):
                rayfish_docker.soak(*self.peers(), 1 / 3600, report, False)
            result = json.loads(report.read_text())
            self.assertEqual(result["status"], "passed")
            self.assertEqual(result["samples"], 1)
            self.assertEqual(result["bytes"], 20)
            self.assertEqual(result["cold_restart_during_discovery_outage"], "blocked")
            self.assertEqual(report.stat().st_mode & 0o777, 0o600)

    def test_mismatch_and_interruption_are_persisted_not_retried(self):
        for failure in (RuntimeError("request failed"), KeyboardInterrupt()):
            with self.subTest(kind=type(failure).__name__), tempfile.TemporaryDirectory() as directory:
                report = Path(directory) / "report.json"
                first, second = self.peers()
                first.request.side_effect = [[200, "hash", 10], failure]
                with patch.object(rayfish_docker.time, "monotonic", side_effect=[100, 100, 101]):
                    with self.assertRaises(type(failure)):
                        rayfish_docker.soak(first, second, 1, report, True)
                result = json.loads(report.read_text())
                self.assertEqual(result["status"], "failed_or_interrupted")
                self.assertEqual(result["samples"], 0)
                self.assertEqual(first.request.call_count, 2)

    def test_long_sampling_gap_invalidates_continuity(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            with patch.object(rayfish_docker.time, "monotonic", side_effect=[100, 201, 202]), \
                 patch.object(rayfish_docker.time, "time", side_effect=[1000, 1091]):
                with self.assertRaises(ValueError):
                    rayfish_docker.soak(*self.peers(), 1, report, True)
            self.assertEqual(json.loads(report.read_text())["status"], "failed_or_interrupted")

    def test_running_report_is_stale_after_heartbeat_deadline_without_rewriting(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            original = '{"status": "running", "samples": 87}\n'
            report.write_text(original)
            os.utime(report, (1000, 1000))
            with patch.object(rayfish_docker.time, "time", return_value=1090):
                self.assertEqual(rayfish_docker.soak_status(report)["status"], "running")
            with patch.object(rayfish_docker.time, "time", return_value=1091):
                result = rayfish_docker.soak_status(report)
            self.assertEqual(result["status"], "stale")
            self.assertEqual(result["samples"], 87)
            self.assertEqual(report.read_text(), original)

    def test_completed_report_remains_completed_and_future_timestamp_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            report.write_text('{"status": "passed"}')
            os.utime(report, (1000, 1000))
            with patch.object(rayfish_docker.time, "time", return_value=2000):
                self.assertEqual(rayfish_docker.soak_status(report)["status"], "passed")
            with patch.object(rayfish_docker.time, "time", return_value=999):
                with self.assertRaises(ValueError):
                    rayfish_docker.soak_status(report)


if __name__ == "__main__":
    unittest.main()
