from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from shellgame.levels.sections.section1 import Level1_9


@dataclass
class _StubState:
    username: str
    workspace: Path
    current_level: str = "1.9"


def _run_in_cwd(tmp_path: Path, cwd: Path, fn: Any) -> Any:
    old = Path.cwd()
    try:
        # pytest runs in a real filesystem; we can chdir freely
        os.chdir(cwd)
        return fn()
    finally:
        os.chdir(old)


def test_level1_9_rejects_when_not_at_home(tmp_path: Path, monkeypatch: Any) -> None:
    level = Level1_9()

    fake_home = tmp_path / "home" / "student"
    (tmp_path / "home" / "student").mkdir(parents=True)

    # Make Path.home() deterministic
    monkeypatch.setattr(Path, "home", lambda: fake_home)

    state = _StubState(username="student", workspace=tmp_path)

    def _validate() -> tuple[bool, str]:
        return level.validate(None, state)

    ok, msg = _run_in_cwd(tmp_path, tmp_path, _validate)
    assert ok is False
    assert "Nejste doma" in msg


def test_level1_9_requires_marker_even_if_at_home(tmp_path: Path, monkeypatch: Any) -> None:
    level = Level1_9()

    fake_home = tmp_path / "home" / "student"
    fake_home.mkdir(parents=True)
    monkeypatch.setattr(Path, "home", lambda: fake_home)

    state = _StubState(username="student", workspace=tmp_path)

    user_dir = Path(f"/tmp/shellgame-{state.username}")
    user_dir.mkdir(parents=True, exist_ok=True)

    marker = user_dir / ".level1_9_cd_walk_completed"
    if marker.exists():
        marker.unlink()

    ok, msg = _run_in_cwd(tmp_path, fake_home, lambda: level.validate(None, state))
    assert ok is False
    assert "nezaznamenal" in msg


def test_level1_9_accepts_submit_without_answer_when_marker_present(tmp_path: Path, monkeypatch: Any) -> None:
    level = Level1_9()

    fake_home = tmp_path / "home" / "student"
    fake_home.mkdir(parents=True)
    monkeypatch.setattr(Path, "home", lambda: fake_home)

    state = _StubState(username="student", workspace=tmp_path)

    user_dir = Path(f"/tmp/shellgame-{state.username}")
    user_dir.mkdir(parents=True, exist_ok=True)
    (user_dir / ".level1_9_cd_walk_completed").write_text("")

    ok, _ = _run_in_cwd(tmp_path, fake_home, lambda: level.validate(None, state))
    assert ok is True

    ok2, _ = _run_in_cwd(tmp_path, fake_home, lambda: level.validate("ignored", state))
    assert ok2 is True
