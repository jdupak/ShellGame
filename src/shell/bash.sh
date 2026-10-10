# ShellGame shell integration for Bash
#
# The launcher (src/shell/launch.rs) starts `bash --rcfile <this file> -i` and hands over everything
# this script needs through the environment, so no path is ever pasted into this text:
#   SHELLGAME_WRAPPER   1: the binary knows it runs inside the game shell
#   SHELLGAME_BIN       absolute path of the shellgame binary
#   SHELLGAME_INIT      path of this script
#   SHELLGAME_FD_HOOK   path of the fd-hook launcher (level scripts execute it)
# The binary itself exports SHELLGAME_WORKSPACE and SHELLGAME_LEVEL through the protocol below.
#
# This file must stay compatible with bash 3.2 (the system bash on macOS): no associative arrays,
# mapfile/readarray, case modification (${x,,}), &>> or |&.

# Disable shell features that can interfere with the game.
unset PROMPT_COMMAND
set +o histexpand
shopt -u checkwinsize 2>/dev/null || true
shopt -s globasciiranges 2>/dev/null || true

@DEV_SHORTCUTS@

# Decode one base64-encoded protocol argument.
__shellgame_decode() {
    printf '%s' "$1" | base64 --decode
}

# Execute one closed protocol operation.
# Arguments: <stderr file> <version> <verb> [<base64 argument>...]
# Uses `builtin cd` to bypass the user-facing cd hook when the game teleports.
__shellgame_eval() {
    local stderr_file="$1"
    [ "$#" -ge 3 ] || return 2
    local version="$2"
    local verb="$3"
    shift 3

    if [ "$version" != "v1" ]; then
        return 2
    fi

    case "$verb" in
        cd)
            [ "$#" -eq 1 ] || return 2
            local destination
            destination="$(__shellgame_decode "$1")" || return 2
            builtin cd -- "$destination"
            ;;
        export)
            [ "$#" -eq 2 ] || return 2
            local name value
            name="$(__shellgame_decode "$1")" || return 2
            value="$(__shellgame_decode "$2")" || return 2
            [[ "$name" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || return 2
            printf -v "$name" '%s' "$value"
            export "$name"
            ;;
        echo)
            [ "$#" -eq 1 ] || return 2
            __shellgame_decode "$1"
            printf '\n'
            ;;
        pwd)
            [ "$#" -eq 0 ] || return 2
            builtin pwd
            ;;
        exit)
            [ "$#" -eq 0 ] || return 2
            command rm -f "$stderr_file"
            builtin exit 0
            ;;
        *)
            return 2
            ;;
    esac
}

# Status handling below uses `cmd || rc=$?` instead of `cmd; rc=$?`, so these functions also work
# when the player has run `set -e` in the shell: a failing command must not end the shell before
# the function has done its job (replayed the output, allowed the move, cleaned up).

pwd() {
    local rc=0
    builtin pwd "$@" || rc=$?
    # Record usage for game verification.
    # The workspace is only ever supplied by the game; never guess it.
    if [ "$rc" -eq 0 ] && [ -n "${SHELLGAME_WORKSPACE:-}" ] && [ -d "$SHELLGAME_WORKSPACE" ]; then
        command touch "$SHELLGAME_WORKSPACE/.pwd_used"
    fi
    return "$rc"
}

# The one `cd` hook for every level. The binary decides from its saved state whether a move is
# allowed, so this function never changes between levels.
cd() {
    local before="$PWD" rc=0

    # Ask the binary whether this move is allowed: `cd-hook pre <pwd> <previous dir> -- <cd args>`.
    # It runs directly, not through `shellgame`, so its explanation reaches the terminal live and
    # nothing is relocated or replayed. Exit status 3 means "blocked"; every other status
    # (0, a crash, a missing binary) allows the move, so a broken hook can never lock the player in.
    "$SHELLGAME_BIN" cd-hook pre "$before" "${OLDPWD:-}" -- "$@" || rc=$?
    if [ "$rc" -eq 3 ]; then
        return 1
    fi

    rc=0
    builtin cd "$@" || rc=$?

    # Report the completed move so the level can track progress. `$PWD` is where we really are.
    if [ "$rc" -eq 0 ]; then
        "$SHELLGAME_BIN" cd-hook post "$before" "$PWD" -- "$@" >/dev/null 2>&1 || true
    fi

    return "$rc"
}

shellgame() {
    local stderr_file
    stderr_file="$(command mktemp)" || return 1

    # If the current working directory was deleted, relocate before running the binary.
    if [ ! -d "$PWD" ]; then
        if [ -n "${SHELLGAME_WORKSPACE:-}" ] && [ -d "$SHELLGAME_WORKSPACE" ]; then
            builtin cd -- "$SHELLGAME_WORKSPACE" 2>/dev/null || builtin cd /tmp 2>/dev/null || true
        else
            builtin cd /tmp 2>/dev/null || true
        fi
    fi

    # stdin and stdout stay on the terminal; only stderr is captured and replayed below.
    # The binary is executed as argv, never via eval/string splitting.
    local exit_code=0
    "$SHELLGAME_BIN" "$@" 2>"$stderr_file" || exit_code=$?

    local line payload
    local -a words
    while IFS= read -r line || [ -n "$line" ]; do
        # Keep exact output except for removing a single trailing CR.
        # Do NOT trim generally; it can corrupt legitimate stderr.
        line="${line%$'\r'}"

        # Only treat lines that start with the marker as directives.
        # This prevents leaking protocol lines to the user.
        if [[ "$line" == __SHELLGAME_EXEC__* ]]; then
            # Split the payload on single spaces without ever evaluating it. The sentinel space
            # keeps an empty last argument (an empty base64 string) as an empty word.
            payload="${line#__SHELLGAME_EXEC__} "
            words=()
            while [ -n "$payload" ]; do
                words[${#words[@]}]="${payload%% *}"
                payload="${payload#* }"
            done
            __shellgame_eval "$stderr_file" "${words[@]}" || true
        else
            printf '%s\n' "$line" >&2
        fi
    done < "$stderr_file"

    command rm -f "$stderr_file"

    # The binary asks for the game shell to close with the `exit` directive (for example after
    # `shellgame remove`), so no command name is special-cased here.
    return "$exit_code"
}

_shellgame_completions()
{
  local cur subcmd
  COMPREPLY=()
  cur="${COMP_WORDS[COMP_CWORD]}"

  local commands="init hint submit skip status reset repeat show remove exit update"

  if [ "$COMP_CWORD" -eq 1 ]; then
    if [[ "${cur}" == -* ]]; then
      COMPREPLY=( $(compgen -W "--help" -- "${cur}") )
    else
      COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") )
    fi
    return 0
  fi

  subcmd="${COMP_WORDS[1]}"
  case "${subcmd}" in
    hint)
      COMPREPLY=( $(compgen -W "-r --repeat --help" -- "${cur}") )
      ;;
    repeat)
      COMPREPLY=( $(compgen -W "--section --level --help" -- "${cur}") )
      ;;
    show)
      COMPREPLY=( $(compgen -W "--section --level --help" -- "${cur}") )
      ;;
    submit)
      COMPREPLY=( $(compgen -f -- "${cur}") )
      ;;
    dev)
      if [ "$COMP_CWORD" -eq 2 ]; then
        COMPREPLY=( $(compgen -W "jump next prev reload start" -- "${cur}") )
      fi
      ;;
    *)
      ;;
  esac
}
complete -F _shellgame_completions shellgame sg g

# Export the functions so `shellgame` also works in child bash processes
# (SHELLGAME_BIN is exported by the launcher). The protocol helpers must travel with it.
export -f __shellgame_decode __shellgame_eval shellgame

# Autostart block (runs once when integration is sourced).
# This must be at the end so the shellgame function is already defined.
if [ -z "${SHELLGAME_AUTO_STARTED:-}" ]; then
    export SHELLGAME_AUTO_STARTED=1
    shellgame
fi
