"""
Shell integration hooks registry + code generators.

This module is a single source of truth for *which* levels require shell-side
instrumentation (hooks), e.g. wrapping `cd` to enforce navigation constraints.

The generated wrapper is deliberately identical for every hooked level: it
forwards the attempted move to Python and does nothing else. All decisions live
in the level itself, so adding a hooked level extends one pattern list instead
of emitting another copy of the wrapper.
"""

from __future__ import annotations

from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry

_BASH_HOOK_BODY = r"""
            # Ask Python whether this move is allowed for the current level.
            local target="${1:-}"
            if ! shellgame cd-hook "$target" "$PWD"; then
                return 1
            fi

            builtin cd "$@"
            local rc=$?

            # Report the completed move so the level can track progress.
            if [ $rc -eq 0 ]; then
                shellgame cd-hook --post-move "$PWD" >/dev/null 2>&1
            fi

            return $rc
"""

_FISH_HOOK_BODY = r"""
            set -l target $argv[1]

            # Ask Python whether this move is allowed for the current level.
            if not shellgame cd-hook "$target" "$PWD"
                return 1
            end

            builtin cd $argv
            set -l rc $status

            # Report the completed move so the level can track progress.
            if test $rc -eq 0
                shellgame cd-hook --post-move "$PWD" >/dev/null 2>&1
            end

            return $rc
"""


def get_cd_hooked_levels() -> frozenset[str]:
    initialize_levels()
    return frozenset(str(level.id) for level in get_registry().list_levels() if "cd" in level.hooks)


def _sorted_hooked_levels() -> list[str]:
    """Hooked level IDs in level order, so the generated script is stable."""

    def key(level_id: str) -> tuple[int, ...]:
        return tuple(int(part) for part in level_id.split("."))

    return sorted(get_cd_hooked_levels(), key=key)


def generate_bash_cd_hooks() -> str:
    levels = _sorted_hooked_levels()
    if not levels:
        return ""

    patterns = "|".join(f'"{level_id}"' for level_id in levels)
    return "\n".join(
        [
            "# Auto-generated cd hook dispatch (ShellGame)",
            "cd() {",
            '    case "$SHELLGAME_LEVEL" in',
            f"        {patterns})",
            _BASH_HOOK_BODY,
            "            ;;",
            '        *) builtin cd "$@" ;;',
            "    esac",
            "}",
            "",
        ]
    )


def generate_fish_cd_hooks() -> str:
    levels = _sorted_hooked_levels()
    if not levels:
        return ""

    patterns = " ".join(f'"{level_id}"' for level_id in levels)
    return "\n".join(
        [
            "# Auto-generated cd hook dispatch (ShellGame)",
            "function cd",
            '    switch "$SHELLGAME_LEVEL"',
            f"        case {patterns}",
            _FISH_HOOK_BODY,
            "        case '*'",
            "            builtin cd $argv",
            "    end",
            "end",
            "",
        ]
    )
