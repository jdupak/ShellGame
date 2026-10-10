#!/usr/bin/env bash
# Mock of the shellgame binary for tests/shell.rs. The tests point SHELLGAME_BIN at a copy of this
# script and drive it through MOCK_* environment variables, so the shell integration can be
# exercised without the real game.
#
# Every call appends records to "$MOCK_LOG". A record is one line whose fields are separated by the
# ASCII unit separator (0x1f), so arguments may contain spaces, quotes and shell metacharacters:
#   argv <arg>...            the arguments of this call (bare `argv` for a call without arguments)
#   cwd <physical cwd>       the working directory the binary was started in
#   env <NAME>=<value>...    a few environment variables ("<unset>" when not set)
#   fds <stdout> <stderr>    fd-hook calls only: whether fds 1 and 2 are /dev/null ("null"/"other")
#   stdin <content>          regular calls with $MOCK_READ_STDIN set: everything read from stdin
#
# Behaviour (all optional):
#   cd-hook pre ...   prints $MOCK_HOOK_STDERR to stderr, exits with $MOCK_PRE_EXIT (default 0)
#   cd-hook post ...  prints $MOCK_POST_NOISE to stdout and stderr, exits with $MOCK_POST_EXIT (default 0)
#   fd-hook ...       exits 0
#   anything else     records stdin if asked to, copies $MOCK_STDOUT_FILE to stdout and
#                     $MOCK_STDERR_FILE to stderr (byte for byte), then exits with $MOCK_EXIT (default 0)

US=$'\x1f'

record() {
    local line="$1" arg
    shift
    for arg in "$@"; do
        line="$line$US$arg"
    done
    printf '%s\n' "$line" >>"${MOCK_LOG:?MOCK_LOG must be set}"
}

record argv "$@"
record cwd "$(pwd -P)"
record env \
    "SHELLGAME_WRAPPER=${SHELLGAME_WRAPPER-<unset>}" \
    "SHELLGAME_BIN=${SHELLGAME_BIN-<unset>}" \
    "SHELLGAME_INIT=${SHELLGAME_INIT-<unset>}" \
    "SHELLGAME_FD_HOOK=${SHELLGAME_FD_HOOK-<unset>}" \
    "SHELLGAME_AUTO_STARTED=${SHELLGAME_AUTO_STARTED-<unset>}" \
    "BASH_SILENCE_DEPRECATION_WARNING=${BASH_SILENCE_DEPRECATION_WARNING-<unset>}"

case "${1-}" in
    cd-hook)
        if [ "${2-}" = post ]; then
            if [ -n "${MOCK_POST_NOISE-}" ]; then
                printf '%s\n' "$MOCK_POST_NOISE"
                printf '%s\n' "$MOCK_POST_NOISE" >&2
            fi
            exit "${MOCK_POST_EXIT-0}"
        fi
        if [ -n "${MOCK_HOOK_STDERR-}" ]; then
            printf '%s\n' "$MOCK_HOOK_STDERR" >&2
        fi
        exit "${MOCK_PRE_EXIT-0}"
        ;;
    fd-hook)
        out=other
        err=other
        [ /dev/stdout -ef /dev/null ] && out=null
        [ /dev/stderr -ef /dev/null ] && err=null
        record fds "$out" "$err"
        exit 0
        ;;
esac

if [ -n "${MOCK_READ_STDIN-}" ]; then
    record stdin "$(cat)"
fi
if [ -n "${MOCK_STDOUT_FILE-}" ]; then
    cat "$MOCK_STDOUT_FILE"
fi
if [ -n "${MOCK_STDERR_FILE-}" ]; then
    cat "$MOCK_STDERR_FILE" >&2
fi
exit "${MOCK_EXIT-0}"
