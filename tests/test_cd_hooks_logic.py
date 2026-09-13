"""Tests for Python-side cd hook logic (GameSession.handle_cd_hook)."""

from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from shellgame.core.session import GameSession
from shellgame.levels.cdpolicy import cd_marker
from shellgame.levels.sections.section1 import AbsoluteCdLevel, HomeWalkLevel, MazeLevel
from shellgame.markers import MarkerManager
from shellgame.state.manager import GameState


@pytest.fixture
def mock_session() -> GameSession:
    """Create a GameSession with mocked dependencies."""
    registry = MagicMock()

    def get_level(level_id: str):
        if level_id == "1.7":
            return MazeLevel()
        if level_id == "1.8":
            return AbsoluteCdLevel()
        if level_id == "1.9":
            return HomeWalkLevel()
        return None

    registry.get.side_effect = get_level

    session = GameSession(
        console=MagicMock(),
        display=MagicMock(),
        state_manager=MagicMock(),
        level_registry=registry,
        teleport_notice=MagicMock(),
        shell_client=MagicMock(),
        workspace_factory=MagicMock(),
    )
    return session


@pytest.fixture
def mock_state(tmp_path: Path) -> GameState:
    """Create a mock GameState."""
    return GameState(
        username="tester",
        workspace=tmp_path,
        current_level="1.0",
        start_time=MagicMock(),
    )


class TestLevel18HookLogic:
    """Test Level 1.8 logic (absolute path detection)."""

    def test_detects_absolute_path(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.8"
        mock_session._state_manager.load.return_value = mock_state

        with patch("shellgame.markers.MarkerManager.create") as mock_create:
            # Act: cd /tmp (absolute)
            mock_session.handle_cd_hook(target="/tmp", pwd="/home", post_move=False)

            # Assert: Marker created
            mock_create.assert_called_with(cd_marker("1.8"))

    def test_rejects_relative_path_from_hook(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.8"
        mock_session._state_manager.load.return_value = mock_state

        with pytest.raises(SystemExit) as excinfo:
            # Act: cd tmp (relative)
            mock_session.handle_cd_hook(target="tmp", pwd="/home", post_move=False)
        assert excinfo.value.code == 1

    def test_ignores_post_move(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.8"
        mock_session._state_manager.load.return_value = mock_state

        with patch("shellgame.markers.MarkerManager.create") as mock_create:
            # Act: post-move
            mock_session.handle_cd_hook(target=None, pwd="/tmp", post_move=True)

            # Assert: Marker NOT created
            mock_create.assert_not_called()

    def test_rejects_relative_path(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.8"
        mock_session._state_manager.load.return_value = mock_state

        with pytest.raises(SystemExit) as excinfo:
            mock_session.handle_cd_hook(target="relative", pwd="/home", post_move=False)
        assert excinfo.value.code == 1

    def test_rejects_empty_target(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.8"
        mock_session._state_manager.load.return_value = mock_state

        with pytest.raises(SystemExit) as excinfo:
            mock_session.handle_cd_hook(target="", pwd="/home", post_move=False)
        assert excinfo.value.code == 1


class TestLevel19HookLogic:
    """Test Level 1.9 logic (step-by-step walk)."""

    def test_pre_move_rejects_jumps(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.9"
        mock_session._state_manager.load.return_value = mock_state

        # Empty target (cd home)
        with pytest.raises(SystemExit) as excinfo:
            mock_session.handle_cd_hook(target="", pwd="/", post_move=False)
        assert excinfo.value.code == 1

        # Absolute path (not root)
        with pytest.raises(SystemExit) as excinfo:
            mock_session.handle_cd_hook(target="/home", pwd="/", post_move=False)
        assert excinfo.value.code == 1

        # Multi-segment relative
        with pytest.raises(SystemExit) as excinfo:
            mock_session.handle_cd_hook(target="a/b", pwd="/", post_move=False)
        assert excinfo.value.code == 1

    def test_pre_move_accepts_root(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.9"
        mock_session._state_manager.load.return_value = mock_state

        # Should not raise
        mock_session.handle_cd_hook(target="/", pwd="/home", post_move=False)

    def test_pre_move_accepts_single_segment(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.9"
        mock_session._state_manager.load.return_value = mock_state

        # Should not raise
        mock_session.handle_cd_hook(target="home", pwd="/", post_move=False)
        mock_session.handle_cd_hook(target="home/", pwd="/", post_move=False)

    def test_starts_tracking_at_root(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.9"
        mock_session._state_manager.load.return_value = mock_state

        with patch("shellgame.markers.MarkerManager.create") as mock_create:
            # Act: cd / (post-move)
            mock_session.handle_cd_hook(target=None, pwd="/", post_move=True)

            # Assert: Progress initialized
            mock_create.assert_called_with(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS, "/")

    def test_tracks_valid_step(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.9"
        mock_session._state_manager.load.return_value = mock_state

        with (
            patch("shellgame.markers.MarkerManager.read") as mock_read,
            patch("shellgame.markers.MarkerManager.create") as mock_create,
        ):
            # Setup: Previously at /
            mock_read.return_value = "/"

            # Act: cd /home (post-move)
            mock_session.handle_cd_hook(target=None, pwd="/home", post_move=True)

            # Assert: Progress updated
            mock_create.assert_called_with(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS, "/\n/home")

    def test_completes_at_home(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.9"
        mock_session._state_manager.load.return_value = mock_state

        # Mock Path.home() to match our destination
        fake_home = Path("/home/tester")

        with (
            patch("shellgame.markers.MarkerManager.read") as mock_read,
            patch("shellgame.markers.MarkerManager.create") as mock_create,
            patch("pathlib.Path.home", return_value=fake_home),
        ):
            # Setup: Previously at /home
            mock_read.return_value = "/\n/home"

            # Act: cd /home/tester (post-move)
            mock_session.handle_cd_hook(target=None, pwd="/home/tester", post_move=True)

            # Assert: Progress updated AND completion marker created
            # Note: create is called twice
            assert mock_create.call_count == 2
            mock_create.assert_any_call(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS, "/\n/home\n/home/tester")
            mock_create.assert_any_call(MarkerManager.LEVEL1_9_CD_WALK_COMPLETED)

    def test_resets_on_invalid_step(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.9"
        mock_session._state_manager.load.return_value = mock_state

        with (
            patch("shellgame.markers.MarkerManager.read") as mock_read,
            patch("shellgame.markers.MarkerManager.remove") as mock_remove,
        ):
            # Setup: Previously at /
            mock_read.return_value = "/"

            # Act: cd /var/log (jump, skipping /var)
            mock_session.handle_cd_hook(target=None, pwd="/var/log", post_move=True)

            # Assert: Progress removed
            mock_remove.assert_called_with(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS)

    def test_ignores_pre_move(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.9"
        mock_session._state_manager.load.return_value = mock_state

        with patch("shellgame.markers.MarkerManager.create") as mock_create:
            # Act: pre-move
            mock_session.handle_cd_hook(target="/", pwd="/home", post_move=False)

            # Assert: Nothing happens
            mock_create.assert_not_called()


class TestLevel17HookLogic:
    """Test Level 1.7 logic (maze boundary guard and autowin)."""

    def test_pre_move_blocks_leaving_maze(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.7"
        mock_session._state_manager.load.return_value = mock_state

        level = MazeLevel()
        level.prepare(mock_state.workspace)

        # cd /etc or outside maze should be blocked
        with pytest.raises(SystemExit) as excinfo:
            mock_session.handle_cd_hook(
                target="/etc",
                pwd=str(mock_state.workspace / "level-1" / "maze" / "entry"),
                post_move=False,
            )
        assert excinfo.value.code == 1

    def test_pre_move_allows_returning_to_start(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.7"
        mock_session._state_manager.load.return_value = mock_state

        level = MazeLevel()
        level.prepare(mock_state.workspace)
        start_dir = mock_state.workspace / "level-1" / "maze" / "entry"

        # Moving to start_dir is always allowed
        mock_session.handle_cd_hook(
            target=str(start_dir),
            pwd=str(mock_state.workspace / "level-1" / "maze" / "nexus"),
            post_move=False,
        )

    def test_pre_move_allows_navigation_within_maze(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.7"
        mock_session._state_manager.load.return_value = mock_state

        level = MazeLevel()
        level.prepare(mock_state.workspace)

        # cd nexus inside maze
        mock_session.handle_cd_hook(
            target="nexus",
            pwd=str(mock_state.workspace / "level-1" / "maze"),
            post_move=False,
        )

    def test_post_move_autowin_on_final(self, mock_session: GameSession, mock_state: GameState) -> None:
        mock_state.current_level = "1.7"
        mock_session._state_manager.load.return_value = mock_state

        level = MazeLevel()
        level.prepare(mock_state.workspace)

        sanctuary = mock_state.workspace / "level-1" / level._SANCTUARY
        with patch("shellgame.markers.MarkerManager.create") as mock_create:
            mock_session.handle_cd_hook(target=None, pwd=str(sanctuary), post_move=True)
            mock_create.assert_called_with(cd_marker("1.7"))

