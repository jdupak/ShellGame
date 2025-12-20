from __future__ import annotations

from shellgame.cli.subshell import get_fish_integration


def test_generated_fish_integration_uses_builtin_cd_fallback() -> None:
    script = get_fish_integration("shellgame")
    # In fish, `cd` is a builtin; `command cd` fails with "Unknown command: cd".
    assert "builtin cd $argv" in script
    assert "command cd $argv" not in script
