# shellcheck shell=bash
# ShellGame integration for bash: `bash --rcfile <this file> -i`; the contract is in src/shell/mod.rs.
# Must stay bash 3.2 compatible (no associative arrays, mapfile, ${x,,}, &>>, |&). Functions use
# `cmd || rc=$?`, not `cmd; rc=$?`, so they keep working when the player has run `set -e`.

# Features that would interfere with the game.
unset PROMPT_COMMAND
set +o histexpand
shopt -u checkwinsize 2>/dev/null || true
shopt -s globasciiranges 2>/dev/null || true

__shellgame_decode() {
    printf '%s' "$1" | base64 --decode
}

# Runs one directive: <stderr file> <version> <verb> [<base64 argument>...]. A closed set, never eval.
__shellgame_eval() {
    local stderr_file="$1" version="${2-}" verb="${3-}" name value
    [ "$#" -ge 3 ] && [ "$version" = v1 ] || return 2
    shift 3
    case "$verb:$#" in
        cd:1)
            value="$(__shellgame_decode "$1")" || return 2
            builtin cd -- "$value" || return
            ;;
        export:2)
            name="$(__shellgame_decode "$1")" || return 2
            value="$(__shellgame_decode "$2")" || return 2
            [[ "$name" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || return 2
            export "$name=$value"
            ;;
        exit:0)
            command rm -f "$stderr_file"
            builtin exit 0
            ;;
        *)
            return 2
            ;;
    esac
}

# Records that the player used `pwd`, for the levels that ask for it. The workspace is only ever
# supplied by the game; never guess it.
pwd() {
    local rc=0
    builtin pwd "$@" || rc=$?
    [ "$rc" -eq 0 ] && [ -d "${SHELLGAME_WORKSPACE:-}" ] && command touch "$SHELLGAME_WORKSPACE/.pwd_used"
    return "$rc"
}

# The one `cd` hook for every level: the binary decides. Status 3 blocks; any other status, a crash
# or a missing binary allows the move, so a broken hook can never lock the player in.
cd() {
    local before="$PWD" rc=0
    "$SHELLGAME_BIN" cd-hook pre "$before" "${OLDPWD:-}" -- "$@" || rc=$?
    [ "$rc" -eq 3 ] && return 1
    rc=0
    builtin cd "$@" || rc=$?
    [ "$rc" -eq 0 ] && { "$SHELLGAME_BIN" cd-hook post "$before" "$PWD" -- "$@" >/dev/null 2>&1 || true; }
    return "$rc"
}

# A typed chmod outside /tmp is refused (status 3); anything else, also a broken binary, runs it.
chmod() {
    local rc=0
    command "$SHELLGAME_BIN" guard -- "$@" || rc=$?
    [ "$rc" -eq 3 ] && return 1
    command chmod "$@"
}

# Runs the binary with stderr captured, then replays it: directive lines are executed, the rest is
# passed on unchanged. The arguments come from the player, so shellcheck sees none.
# shellcheck disable=SC2120
shellgame() {
    local stderr_file line payload exit_code=0
    local -a words
    stderr_file="$(command mktemp)" || return 1

    "$SHELLGAME_BIN" "$@" 2>"$stderr_file" || exit_code=$?

    # `exit` removes the file while it is read; the open descriptor stays valid.
    # shellcheck disable=SC2094
    while IFS= read -r line || [ -n "$line" ]; do
        line="${line%$'\r'}" # exactly one CR; stderr is otherwise replayed as it is
        if [[ "$line" == __SHELLGAME_EXEC__* ]]; then
            # Split on single spaces, never evaluate. The sentinel space keeps an empty last argument.
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
    return "$exit_code"
}

# Sets COMPREPLY to the lines a command prints, one word each, spaces kept (bash 3.2 has no mapfile).
__shellgame_reply() {
    local word
    COMPREPLY=()
    while IFS= read -r word; do COMPREPLY+=("$word"); done < <("$@")
}

_shellgame_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}" opts="--help"
    if [ "$COMP_CWORD" -eq 1 ]; then
        case "$cur" in
            -*) ;;
            *) opts="hint submit status reset show levels resume remove exit update" ;;
        esac
    elif [ "${COMP_WORDS[1]}" = submit ]; then
        __shellgame_reply compgen -f -- "$cur"
        return 0
    fi
    __shellgame_reply compgen -W "$opts" -- "$cur"
}
complete -F _shellgame_completions shellgame

# Child bash processes can call `shellgame` too (SHELLGAME_BIN is exported by the launcher).
export -f __shellgame_decode __shellgame_eval shellgame

# Autostart: last, so that everything above is defined.
if [ -z "${SHELLGAME_AUTO_STARTED:-}" ]; then
    export SHELLGAME_AUTO_STARTED=1
    shellgame
fi
