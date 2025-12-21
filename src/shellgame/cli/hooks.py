"""
Shell integration hooks registry + code generators.

This module is a single source of truth for *which* levels require shell-side
instrumentation (hooks), e.g. wrapping `cd` to enforce navigation constraints.

The hook registry is also used to generate per-shell hook code blocks which are
injected into the shell integration templates.

Notes:
- Generators return shell code snippets (bash/fish).
- The snippets are meant to be inserted into templates at `$cd_hook`.
- Keep user-facing strings short (Czech) and consistent with existing UX.
"""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class LevelCdHook:
    """Marker for a level that needs a `cd` wrapper/hook in the subshell."""

    level_id: str


# Levels that require a `cd` wrapper in the subshell.
_CD_HOOK_LEVELS: frozenset[str] = frozenset(
    {
        "1.8",  # requires absolute path to a specific target
        "1.9",  # requires walking to $HOME step-by-step
    }
)


def get_hooked_levels() -> frozenset[str]:
    """Return all level IDs that require any shell hook."""
    # Today it's only cd-hooks; later we may union multiple hook sets here.
    return _CD_HOOK_LEVELS


def get_cd_hooked_levels() -> frozenset[str]:
    """Return level IDs that require the `cd` wrapper/hook."""
    return _CD_HOOK_LEVELS


def iter_cd_hooks() -> Iterable[LevelCdHook]:
    """Iterate structured cd hook descriptors (for future codegen use)."""
    # Deterministic order is useful for test stability and future generation.
    for level_id in sorted(_CD_HOOK_LEVELS):
        yield LevelCdHook(level_id=level_id)


def is_level_hooked(level_id: str) -> bool:
    """Convenience predicate: does this level need any shell hook?"""
    return level_id in get_hooked_levels()


def generate_bash_cd_hook(level_id: str) -> str:
    """Generate bash `cd()` wrapper for the given level, or empty string if none."""
    if not is_level_hooked(level_id):
        return ""

    # The logic has been moved to Python (shellgame cd-hook).
    # The shell hook now just calls back into Python to validate the move.
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
    """Generate fish `cd` wrapper for the given level, or empty string if none."""
    if not is_level_hooked(level_id):
        return ""

    # The logic has been moved to Python (shellgame cd-hook).
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
    """Generate a combined bash hook block that selects by $SHELLGAME_LEVEL."""
    # We generate exactly one `cd()` wrapper that dispatches to per-level logic.
    # This makes it safe to inject into templates that default-provide cd().
    
    # Since we moved logic to Python, we can use a generic hook for all hooked levels.
    # But we still need to dispatch based on level ID to only hook relevant levels.
    
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
    
    # Add cases for all hooked levels
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
    """Generate a combined fish hook block that selects by $SHELLGAME_LEVEL."""
    
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
