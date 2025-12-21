"""Unit tests for workspace management."""

from pathlib import Path

from shellgame.workspace.builder import DirSpec, FileSpec, WorkspaceManager


class TestFileSpec:
    """Test FileSpec."""

    def test_create_file_spec(self) -> None:
        """Test creating a file specification."""
        spec = FileSpec("test.txt", "content", 0o644)

        assert spec.path == "test.txt"
        assert spec.content == "content"
        assert spec.mode == 0o644


class TestDirSpec:
    """Test DirSpec."""

    def test_create_dir_spec(self) -> None:
        """Test creating a directory specification."""
        spec = DirSpec("testdir", 0o755)

        assert spec.path == "testdir"
        assert spec.mode == 0o755


class TestWorkspaceManager:
    """Test WorkspaceManager."""

    def test_init_workspace(self, tmp_path: Path) -> None:
        """Test initializing workspace."""
        # Use tmp_path for testing
        manager = WorkspaceManager("testuser")
        manager.workspace_root = tmp_path / "workspace"

        result = manager.init()

        assert result.exists()
        assert result.is_dir()

    def test_build_structure(self, tmp_path: Path) -> None:
        """Test building directory structure."""
        manager = WorkspaceManager("testuser")
        manager.workspace_root = tmp_path / "workspace"
        manager.init()

        dirs = [DirSpec("level-1"), DirSpec("level-1/alpha")]

        files = [FileSpec("level-1/alpha/test.txt", "content")]

        manager.build_structure(dirs, files)

        assert (manager.workspace_root / "level-1").exists()
        assert (manager.workspace_root / "level-1" / "alpha").exists()
        assert (manager.workspace_root / "level-1" / "alpha" / "test.txt").exists()

        content = (manager.workspace_root / "level-1" / "alpha" / "test.txt").read_text()
        assert content == "content"

    def test_validate_integrity(self, tmp_path: Path) -> None:
        """Test workspace integrity validation."""
        manager = WorkspaceManager("testuser")
        manager.workspace_root = Path("/tmp/shellgame-testuser")

        # Create workspace
        manager.init()

        # Should be valid (starts with /tmp/)
        assert manager.validate_integrity() is True

        # Cleanup
        manager.cleanup()

    def test_cleanup(self, tmp_path: Path) -> None:
        """Test workspace cleanup."""
        manager = WorkspaceManager("testuser")
        manager.workspace_root = tmp_path / "workspace"
        manager.init()

        assert manager.workspace_root.exists()

        manager.cleanup()

        assert not manager.workspace_root.exists()

    def test_clear_directory(self, tmp_path: Path) -> None:
        """Test clearing a directory."""
        manager = WorkspaceManager("testuser")
        manager.workspace_root = tmp_path / "workspace"
        manager.init()

        # Create some structure
        test_dir = manager.workspace_root / "testdir"
        test_dir.mkdir()
        (test_dir / "file.txt").write_text("content")

        # Clear the directory
        manager.clear_directory("testdir")

        # Directory should exist but be empty
        assert test_dir.exists()
        assert len(list(test_dir.iterdir())) == 0
