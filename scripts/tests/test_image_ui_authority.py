import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class ImageUiAuthorityTests(unittest.TestCase):
    def read(self, path):
        return (ROOT / path).read_text()

    def test_ui_assets_always_use_the_embedded_application_image(self):
        source = self.read("rust-arr/src/ui_assets.rs")
        self.assertIn("embedded.to_owned()", source)
        self.assertIn("embedded.to_vec()", source)
        self.assertIn("always_serves_embedded_assets_regardless_of_bundle_path", source)

    def test_container_startup_and_dockerfile_have_no_ui_updater(self):
        entrypoint = self.read("rust-arr/container-entrypoint.sh")
        dockerfile = self.read("rust-arr/Dockerfile")
        self.assertNotIn("rawrarr-update-ui", entrypoint)
        self.assertNotIn("RAWARR_UI_GITHUB", entrypoint)
        self.assertNotIn("update_ui_bundle", dockerfile)
        self.assertFalse((ROOT / "rust-arr/update_ui_bundle.py").exists())

    def test_ui_changes_trigger_application_image_publish(self):
        tests_workflow = self.read(".github/workflows/tests.yml")
        image_workflow = self.read(".github/workflows/publish-container.yml")
        self.assertNotIn("Publish private UI bundle", tests_workflow)
        self.assertNotIn("package_ui_bundle", tests_workflow)
        self.assertIn("- 'rust-arr/**'", image_workflow)

    def test_coordinator_compose_no_longer_requires_a_ui_github_token(self):
        compose = self.read("deploy/do3/docker-compose.internal-coordinator.yml")
        self.assertNotIn("RAWARR_UI_GITHUB", compose)
        self.assertNotIn("rawrarr_ui_github_token", compose)


if __name__ == "__main__":
    unittest.main()
