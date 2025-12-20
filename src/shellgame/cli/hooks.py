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

from dataclasses import dataclass
from typing import FrozenSet, Iterable


@dataclass(frozen=True, slots=True)
class LevelCdHook:
    """Marker for a level that needs a `cd` wrapper/hook in the subshell."""

    level_id: str


# Levels that require a `cd` wrapper in the subshell.
_CD_HOOK_LEVELS: FrozenSet[str] = frozenset(
    {
        "1.8",  # requires absolute path to a specific target
        "1.9",  # requires walking to $HOME step-by-step
    }
)


def get_hooked_levels() -> FrozenSet[str]:
    """Return all level IDs that require any shell hook."""
    # Today it's only cd-hooks; later we may union multiple hook sets here.
    return _CD_HOOK_LEVELS


def get_cd_hooked_levels() -> FrozenSet[str]:
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
    if level_id == "1.8":
        return r"""
cd() {
    if [ $# -eq 0 ]; then
        echo "ShellGame (1.8): V tomhle levelu použijte absolutní cestu (začíná na /)." >&2
        return 1
    fi

    local target="$1"
    if [[ ! "$target" =~ ^/ ]]; then
        local abs_target="$SHELLGAME_WORKSPACE/level-1/absolute-target"
        local candidate=""
        if command -v realpath >/dev/null 2>&1; then
            candidate="$(realpath -m "$target" 2>/dev/null)"
        fi
        if [ -n "$candidate" ] && [ "$candidate" = "$abs_target" ]; then
            echo "ShellGame (1.8): Tohle by fungovalo, ale je to relativní cesta. Zkuste to ještě jednou absolutně (začíná na /)." >&2
            echo "Tip: cd $abs_target" >&2
        else
            echo "ShellGame (1.8): Použijte absolutní cestu (začíná na /)." >&2
        fi
        return 1
    fi

    builtin cd "$@"
    local rc=$?

    local abs_target="$SHELLGAME_WORKSPACE/level-1/absolute-target"
    if [ $rc -eq 0 ] && [ "$(pwd)" = "$abs_target" ]; then
        local user_dir="/tmp/shellgame-$USER"
        if [ -d "$user_dir" ]; then
            touch "$user_dir/.level1_8_absolute_cd_used"
        fi
    fi

    return $rc
}
""".lstrip("\n")

    if level_id == "1.9":
        return r"""
cd() {
    local user_dir="/tmp/shellgame-$USER"
    if [ ! -d "$user_dir" ]; then
        builtin cd "$@"
        return $?
    fi

    local progress_file="$user_dir/.level1_9_cd_walk_progress"
    local completed_file="$user_dir/.level1_9_cd_walk_completed"

    local home_real="$HOME"
    if command -v realpath >/dev/null 2>&1; then
        home_real="$(realpath -m "$HOME" 2>/dev/null)"
    fi

    local progress=0
    if [ -f "$progress_file" ]; then
        local raw
        raw="$(tr -d ' \t\n\r' < "$progress_file" 2>/dev/null)"
        if [[ "$raw" =~ ^[0-9]+$ ]]; then
            progress="$raw"
        fi
    fi

    if [ $# -eq 0 ]; then
        echo "ShellGame (1.9): V tomhle levelu nepoužívejte prázdné 'cd'. Začněte: cd /" >&2
        return 1
    fi

    local target="$1"

    if [ "$target" = "/" ]; then
        builtin cd /
        echo 0 > "$progress_file"
        rm -f "$completed_file"
        return $?
    fi

    if [[ "$target" == ~* ]]; then
        echo "ShellGame (1.9): Zkratky na domov ('~') teď nepoužívejte. Jděte krok za krokem: cd / ; cd <segment> ..." >&2
        return 1
    fi
    # Detect expanded ~ (bash expands ~ before calling the function)
    if [ "$target" = "$HOME" ] || [ "$target" = "$home_real" ]; then
        echo "ShellGame (1.9): Zkratky na domov ('~') teď nepoužívejte. Jděte krok za krokem: cd / ; cd <segment> ..." >&2
        return 1
    fi
    if [[ "$target" =~ ^/ ]]; then
        echo "ShellGame (1.9): Nepřeskakujte absolutní cestou. Začněte v / a vstupujte do každého segmentu zvlášť." >&2
        return 1
    fi
    if [[ "$target" == */* ]]; then
        echo "ShellGame (1.9): Nepřeskakujte více úrovní najednou. Použijte jen jeden segment (bez '/')." >&2
        return 1
    fi
    if [[ "$target" == ..* ]]; then
        echo "ShellGame (1.9): Teď necvičíme návrat zpět. Pokud jste se ztratili, dejte: cd /" >&2
        return 1
    fi

    local trimmed="${home_real#/}"
    local IFS='/'
    read -r -a segments <<< "$trimmed"

    if [ "$progress" -eq 0 ] && [ "$(pwd)" != "/" ]; then
        echo "ShellGame (1.9): Nejdřív musíte být v /. Použijte: cd /" >&2
        return 1
    fi

    local expected_prefix="/"
    if [ "$progress" -gt 0 ]; then
        expected_prefix="/$(printf "%s/" "${segments[@]:0:$progress}" | sed 's:/$::')"
    fi
    if [ "$(pwd)" != "$expected_prefix" ]; then
        echo "ShellGame (1.9): Jste v nesprávné části cesty. Očekávám: $expected_prefix. Reset: cd /" >&2
        return 1
    fi

    local next_index=$((progress + 1))
    local total=${#segments[@]}
    if [ "$next_index" -gt "$total" ]; then
        touch "$completed_file"
        return 0
    fi

    local expected_next="${segments[$((next_index - 1))]}"
    if [ "$target" != "$expected_next" ]; then
        echo "ShellGame (1.9): Teď je na řadě segment '$expected_next', ne '$target'. Pokud nevíte kudy, dejte: echo \$HOME" >&2
        return 1
    fi

    builtin cd -- "$target"
    local rc=$?
    if [ $rc -ne 0 ]; then
        return $rc
    fi

    echo "$next_index" > "$progress_file"
    if [ "$(pwd)" = "$home_real" ]; then
        touch "$completed_file"
    fi

    return 0
}
""".lstrip("\n")

    return ""


def generate_fish_cd_hook(level_id: str) -> str:
    """Generate fish `cd` wrapper for the given level, or empty string if none."""
    if level_id == "1.8":
        return r"""
function cd
    set -l arg_count (count $argv)
    if test $arg_count -eq 0
        echo "ShellGame (1.8): V tomhle levelu použijte absolutní cestu (začíná na /)." >&2
        return 1
    end

    set -l target $argv[1]
    if not string match -rq '^/' -- $target
        set -l abs_target "$SHELLGAME_WORKSPACE/level-1/absolute-target"
        set -l candidate (realpath -m "$target" 2>/dev/null)
        if test "$candidate" = "$abs_target"
            echo "ShellGame (1.8): Tohle by fungovalo, ale je to relativní cesta. Zkuste to ještě jednou absolutně (začíná na /)." >&2
            echo "Tip: cd $abs_target" >&2
        else
            echo "ShellGame (1.8): Použijte absolutní cestu (začíná na /)." >&2
        end
        return 1
    end

    builtin cd $argv
    set -l rc $status

    set -l abs_target "$SHELLGAME_WORKSPACE/level-1/absolute-target"
    if test $rc -eq 0; and test (pwd) = "$abs_target"
        set -l user_dir "/tmp/shellgame-$USER"
        if test -d "$user_dir"
            touch "$user_dir/.level1_8_absolute_cd_used"
        end
    end

    return $rc
end
""".lstrip("\n")

    if level_id == "1.9":
        return r"""
function cd
    set -l user_dir "/tmp/shellgame-$USER"
    if not test -d "$user_dir"
        builtin cd $argv
        return $status
    end

    set -l progress_file "$user_dir/.level1_9_cd_walk_progress"
    set -l completed_file "$user_dir/.level1_9_cd_walk_completed"

    set -l home_real (realpath -m "$HOME" 2>/dev/null)
    if test -z "$home_real"
        set home_real "$HOME"
    end

    set -l home_segments (string split "/" -- (string trim -c "/" -- "$home_real"))
    if test -z "$home_segments[1]"
        set home_segments
    end

    set -l progress 0
    if test -f "$progress_file"
        set -l raw (string trim -- (cat "$progress_file" 2>/dev/null))
        if string match -rq '^[0-9]+$' -- $raw
            set progress $raw
        end
    end

    set -l argc (count $argv)
    if test $argc -eq 0
        echo "ShellGame (1.9): V tomhle levelu nepoužívejte prázdné 'cd'. Začněte: cd /" >&2
        return 1
    end

    set -l target $argv[1]

    if test "$target" = "/"
        builtin cd /
        echo 0 > "$progress_file"
        rm -f "$completed_file"
        return $status
    end

    if test "$target" = "~"; or string match -rq '^~' -- "$target"
        echo "ShellGame (1.9): Zkratky na domov ('~') teď nepoužívejte. Jděte krok za krokem: cd / ; cd <segment> ..." >&2
        return 1
    end
    if string match -rq '^/' -- "$target"
        echo "ShellGame (1.9): Nepřeskakujte absolutní cestou. Začněte v / a vstupujte do každého segmentu zvlášť." >&2
        return 1
    end
    if string match -q '*/*' -- "$target"
        echo "ShellGame (1.9): Nepřeskakujte více úrovní najednou. Použijte jen jeden segment (bez '/')." >&2
        return 1
    end
    if test "$target" = ".."; or string match -q '..*' -- "$target"
        echo "ShellGame (1.9): Teď necvičíme návrat zpět. Pokud jste se ztratili, dejte: cd /" >&2
        return 1
    end

    if test $progress -eq 0
        if test (pwd) != "/"
            echo "ShellGame (1.9): Nejdřív musíte být v /. Použijte: cd /" >&2
            return 1
        end
    end

    set -l expected_prefix "/"
    if test $progress -gt 0
        set expected_prefix "/"(string join "/" -- $home_segments[1..$progress])
    end
    if test (pwd) != "$expected_prefix"
        echo "ShellGame (1.9): Jste v nesprávné části cesty. Očekávám: $expected_prefix. Reset: cd /" >&2
        return 1
    end

    set -l next_index (math $progress + 1)
    if test $next_index -gt (count $home_segments)
        touch "$completed_file"
        return 0
    end

    set -l expected_next $home_segments[$next_index]
    if test "$target" != "$expected_next"
        echo "ShellGame (1.9): Teď je na řadě segment '$expected_next', ne '$target'. Pokud nevíte kudy, dejte: echo $HOME" >&2
        return 1
    end

    builtin cd -- "$target"
    if test $status -ne 0
        return $status
    end

    echo $next_index > "$progress_file"
    if test (pwd) = "$home_real"
        touch "$completed_file"
    end
    return 0
end
""".lstrip("\n")

    return ""


def generate_bash_cd_hooks() -> str:
    """Generate a combined bash hook block that selects by $SHELLGAME_LEVEL."""
    # We generate exactly one `cd()` wrapper that dispatches to per-level logic.
    # This makes it safe to inject into templates that default-provide cd().
    parts: list[str] = []
    for level_id in sorted(get_cd_hooked_levels()):
        hook = generate_bash_cd_hook(level_id)
        if hook:
            # Wrap each per-level cd() into a level-specific function to avoid redefinition,
            # then dispatch from a single cd().
            fn_name = f"__shellgame_cd_{level_id.replace('.', '_')}"
            # Convert "cd() {" to "__shellgame_cd_X() {"
            hook_lines = hook.splitlines()
            if hook_lines:
                hook_lines[0] = f"{fn_name}() {{"
            hook = "\n".join(hook_lines)
            parts.append(hook)

    if not parts:
        return ""

    dispatch_lines = [
        "# Auto-generated cd hook dispatch (ShellGame)",
        *parts,
        "",
        "cd() {",
        '    case "$SHELLGAME_LEVEL" in',
    ]
    for level_id in sorted(get_cd_hooked_levels()):
        fn_name = f"__shellgame_cd_{level_id.replace('.', '_')}"
        dispatch_lines.append(f'        "{level_id}") {fn_name} "$@" ;;')
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
    parts: list[str] = []
    for level_id in sorted(get_cd_hooked_levels()):
        hook = generate_fish_cd_hook(level_id)
        if hook:
            fn_name = f"__shellgame_cd_{level_id.replace('.', '_')}"
            hook_lines = hook.splitlines()
            if hook_lines:
                hook_lines[0] = f"function {fn_name}"
            hook = "\n".join(hook_lines)
            parts.append(hook)

    if not parts:
        return ""

    lines: list[str] = []
    lines.append("# Auto-generated cd hook dispatch (ShellGame)")
    lines.extend(parts)
    lines.append("")
    lines.append("function cd")
    lines.append('    switch "$SHELLGAME_LEVEL"')
    for level_id in sorted(get_cd_hooked_levels()):
        fn_name = f"__shellgame_cd_{level_id.replace('.', '_')}"
        lines.append(f'        case "{level_id}"')
        lines.append(f"            {fn_name} $argv")
    lines.append("        case '*'")
    lines.append("            builtin cd $argv")
    lines.append("    end")
    lines.append("end")
    lines.append("")
    return "\n".join(lines)
