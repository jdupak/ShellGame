"""Unit tests for CLI helper functions.

These tests lock in path-mapping behavior so it doesn't regress.
"""

from pathlib import Path

from shellgame.cli.commands import get_level_start_directory
from shellgame.levels.sections.section1 import Level1_1


def test_get_level_start_directory_selected_levels(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"

    # Intro levels start in the workspace root.
    assert get_level_start_directory("0.0", workspace) == workspace

    # A representative "starts somewhere specific" level.
    assert (
        get_level_start_directory("1.4", workspace) == workspace / "level-1" / "alpha"
    )

    # 1.5 is a navigation task; it must not start at the goal directory.
    assert get_level_start_directory("1.5", workspace) == workspace / "level-1"

    # Section intro levels ("x.0") default to "no forced start dir".
    assert get_level_start_directory("2.0", workspace) is None

    # Unknown levels also default to "no forced start dir".
    assert get_level_start_directory("999.9", workspace) is None


def test_level1_1_requires_pwd_marker(tmp_path: Path, monkeypatch) -> None:
    """Level 1.1 should require evidence that `pwd` was used."""

    # Fake state object with just the fields Level1_1.validate expects.
    class _State:
        def __init__(self, username: str, workspace: Path) -> None:
            self.username = username
            self.workspace = workspace

    username = "testuser"
    state = _State(username=username, workspace=tmp_path / "workspace")

    # Make /tmp/shellgame-testuser point inside our tmp dir.
    fake_tmp_shellgame = tmp_path / "tmp_shellgame" / f"shellgame-{username}"
    fake_tmp_shellgame.mkdir(parents=True)
    monkeypatch.setattr(
        "shellgame.levels.sections.section1.Path",
        lambda p="": Path(str(p)).__class__(str(p)),
        raising=False,
    )

    # A lighter-weight approach: monkeypatch the pwd_marker path computation via cwd
    # isn't practical, so we create the real marker file in /tmp.
    # This test assumes the test environment can write to /tmp.
    marker = Path(f"/tmp/shellgame-{username}") / ".pwd_used"
    if marker.exists():
        marker.unlink()

    level = Level1_1()

    ok, msg = level.validate("level-1", state)
    assert ok is False
    assert "pwd" in msg

    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.write_text("")
    try:
        ok2, _ = level.validate("level-1", state)
        assert ok2 is True
    finally:
        marker.unlink(missing_ok=True)


def test_level1_1_wrong_answer_does_not_spoil_expected(tmp_path: Path) -> None:
    """If the pwd marker exists, a wrong answer should fail without revealing the expected value."""

    class _State:
        def __init__(self, username: str, workspace: Path) -> None:
            self.username = username
            self.workspace = workspace

    username = "testuser"
    state = _State(username=username, workspace=tmp_path / "workspace")

    marker = Path(f"/tmp/shellgame-{username}") / ".pwd_used"
    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.write_text("")

    level = Level1_1()
    try:
        ok, msg = level.validate("shellgame-viper", state)
        assert ok is False
        assert "Očekáváno" not in msg
        assert "level-1" not in msg
    finally:
        marker.unlink(missing_ok=True)
