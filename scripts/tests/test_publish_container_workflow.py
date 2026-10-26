from pathlib import Path
import unittest


WORKFLOW = Path(__file__).resolve().parents[2] / ".github/workflows/publish-container.yml"


class PublishContainerWorkflowTests(unittest.TestCase):
    def test_architectures_build_independently_and_manifest_waits_for_both(self):
        source = WORKFLOW.read_text()
        self.assertIn("name: Build ${{ matrix.arch }} image", source)
        self.assertIn("arch: amd64\n            platform: linux/amd64", source)
        self.assertIn("arch: arm64\n            platform: linux/arm64", source)
        self.assertIn("platforms: ${{ matrix.platform }}", source)
        self.assertNotIn("platforms: linux/amd64,linux/arm64", source)
        self.assertIn("needs: build", source)
        self.assertIn(":sha-${{ github.sha }}-${{ matrix.arch }}", source)

    def test_only_manifest_job_publishes_consumer_tags_after_both_builds(self):
        source = WORKFLOW.read_text()
        self.assertIn("name: Publish multi-architecture main and commit tags", source)
        self.assertIn('--tag "$IMAGE:main"', source)
        self.assertIn('--tag "$IMAGE:sha-$COMMIT_SHA"', source)
        self.assertIn('"$IMAGE:sha-$COMMIT_SHA-amd64"', source)
        self.assertIn('"$IMAGE:sha-$COMMIT_SHA-arm64"', source)
        self.assertIn("packages: write", source)
        self.assertIn("cancel-in-progress: false", source)


if __name__ == "__main__":
    unittest.main()
