"""Tests for the generated `cd` hook dispatch.

The wrapper body is identical for every hooked level, so what matters is that
both shells dispatch the same set of levels into it and that the body is emitted
only once.
"""

import pytest

from shellgame.cli.hooks import (
    generate_bash_cd_hooks,
    generate_fish_cd_hooks,
    get_cd_hooked_levels,
)


class TestCdHookGeneration:
    """The shipped dispatch must cover exactly the levels that declare a hook."""

    @pytest.mark.parametrize("level_id", ["1.7", "1.8", "1.9"])
    def test_hooked_levels_are_dispatched_in_both_shells(self, level_id: str) -> None:
        assert level_id in get_cd_hooked_levels()
        assert f'"{level_id}"' in _bash_patterns()
        assert f'"{level_id}"' in _fish_patterns()

    @pytest.mark.parametrize("level_id", ["1.1", "2.0"])
    def test_unhooked_levels_fall_through_to_builtin_cd(self, level_id: str) -> None:
        assert level_id not in get_cd_hooked_levels()
        assert f'"{level_id}"' not in _bash_patterns()
        assert f'"{level_id}"' not in _fish_patterns()

    def test_the_wrapper_body_is_emitted_once(self) -> None:
        """One shared body; adding a hooked level must not grow the script."""
        assert generate_bash_cd_hooks().count("shellgame cd-hook --post-move") == 1
        assert generate_fish_cd_hooks().count("shellgame cd-hook --post-move") == 1


def _bash_patterns() -> str:
    return next(
        line
        for line in generate_bash_cd_hooks().splitlines()
        if line.strip().startswith('"') and line.strip().endswith(")")
    )


def _fish_patterns() -> str:
    line = next(line for line in generate_fish_cd_hooks().splitlines() if line.strip().startswith('case "'))
    return line
