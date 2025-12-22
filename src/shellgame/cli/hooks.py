"""
Shell integration hooks registry + code generators.

This module is a single source of truth for *which* levels require shell-side
instrumentation (hooks), e.g. wrapping `cd` to enforce navigation constraints.
"""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class LevelCdHook:
    level_id: str


_CD_HOOK_LEVELS: frozenset[str] = frozenset(
    {
        "1.8",
        "1.9",
    }
)


def get_hooked_levels() -> frozenset[str]:
    return _CD_HOOK_LEVELS


def get_cd_hooked_levels() -> frozenset[str]:
    return _CD_HOOK_LEVELS


def iter_cd_hooks() -> Iterable[LevelCdHook]:
    for level_id in sorted(_CD_HOOK_LEVELS):
        yield LevelCdHook(level_id=level_id)


def is_level_hooked(level_id: str) -> bool:
    return level_id in get_hooked_levels()


def generate_bash_cd_hook(level_id: str) -> str:
    if not is_level_hooked(level_id):
        return ""

    return r"""
cd() {
    # Call Python to validate the move
    # We pass the target directory (or empty string)
    local target="${1:-}"
    
    # Capture stderr to avoid leaking protocol messages if something goes wrong
    # But we want to see the output of the validation command
    if ! shellgame cd-hook "$target" "$PWD"; then
        return 1
    fi

    # If validation passed, perform the cd
    builtin cd "$@"
    local rc=$?
    
    # Notify Python about successful move (for state tracking)
    if [ $rc -eq 0 ]; then
        shellgame cd-hook --post-move "$PWD" >/dev/null 2>&1
    fi
    
    return $rc
}
""".lstrip("\n")


def generate_fish_cd_hook(level_id: str) -> str:
    if not is_level_hooked(level_id):
        return ""

    return r"""
function cd
    set -l target $argv[1]
    
    # Call Python to validate the move
    if not shellgame cd-hook "$target" "$PWD"
        return 1
    end

    # If validation passed, perform the cd
    builtin cd $argv
    set -l rc $status
    
    # Notify Python about successful move (for state tracking)
    if test $rc -eq 0
        shellgame cd-hook --post-move "$PWD" >/dev/null 2>&1
    end
    
    return $rc
end
""".lstrip("\n")


def generate_bash_cd_hooks() -> str:
    hook_body = r"""
            # Call Python to validate the move
            # We pass the target directory (or empty string)
            local target="${1:-}"
            
            # Capture stderr to avoid leaking protocol messages if something goes wrong
            # But we want to see the output of the validation command
            if ! shellgame cd-hook "$target" "$PWD"; then
                return 1
            fi

            # If validation passed, perform the cd
            builtin cd "$@"
            local rc=$?
            
            # Notify Python about successful move (for state tracking)
            if [ $rc -eq 0 ]; then
                shellgame cd-hook --post-move "$PWD" >/dev/null 2>&1
            fi
            
            return $rc
"""

    dispatch_lines = [
        "# Auto-generated cd hook dispatch (ShellGame)",
        "cd() {",
        '    case "$SHELLGAME_LEVEL" in',
    ]
    
    for level_id in sorted(get_cd_hooked_levels()):
        dispatch_lines.append(f'        "{level_id}")')
        dispatch_lines.append(hook_body)
        dispatch_lines.append('            ;;')
            
    dispatch_lines.extend(
        [
            '        *) builtin cd "$@" ;;',
            "    esac",
            "}",
            "",
        ]
    )
    return "\n".join(dispatch_lines)


def generate_fish_cd_hooks() -> str:
    hook_body = r"""
            set -l target $argv[1]
            
            # Call Python to validate the move
            if not shellgame cd-hook "$target" "$PWD"
                return 1
            end

            # If validation passed, perform the cd
            builtin cd $argv
            set -l rc $status
            
            # Notify Python about successful move (for state tracking)
            if test $rc -eq 0
                shellgame cd-hook --post-move "$PWD" >/dev/null 2>&1
            end
            
            return $rc
"""

    lines: list[str] = []
    lines.append("# Auto-generated cd hook dispatch (ShellGame)")
    lines.append("function cd")
    lines.append('    switch "$SHELLGAME_LEVEL"')
    
    for level_id in sorted(get_cd_hooked_levels()):
        lines.append(f'        case "{level_id}"')
        lines.append(hook_body)
        
    lines.append("        case '*'")
    lines.append("            builtin cd $argv")
    lines.append("    end")
    lines.append("end")
    lines.append("")
    return "\n".join(lines)
