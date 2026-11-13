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

    def test_each_image_build_runs_on_a_native_architecture_runner(self):
        source = WORKFLOW.read_text()
        self.assertIn("runs-on: ${{ matrix.runner }}", source)
        self.assertIn("runner: ubuntu-latest\n            runner_arch: x86_64", source)
        self.assertIn("runner: ubuntu-24.04-arm\n            runner_arch: aarch64", source)
        self.assertIn('test "$(uname -m)" = "${{ matrix.runner_arch }}"', source)
        self.assertNotIn("docker/setup-qemu-action", source)

    def test_only_manifest_job_publishes_consumer_tags_after_both_builds(self):
        source = WORKFLOW.read_text()
        self.assertIn("name: Publish multi-architecture main and commit tags", source)
        self.assertIn('--tag "$IMAGE:main"', source)
        self.assertIn('--tag "$IMAGE:sha-$COMMIT_SHA"', source)
        self.assertIn('"$IMAGE:sha-$COMMIT_SHA-amd64"', source)
        self.assertIn('"$IMAGE:sha-$COMMIT_SHA-arm64"', source)
        self.assertIn("packages: write", source)
        self.assertIn("cancel-in-progress: false", source)

    def test_binary_build_revision_matches_the_oci_revision_label(self):
        source = WORKFLOW.read_text()
        self.assertIn("build-args: |\n            RAWARR_BUILD_REVISION=${{ github.sha }}", source)
        self.assertIn("org.opencontainers.image.revision=${{ github.sha }}", source)


if __name__ == "__main__":
    unittest.main()
