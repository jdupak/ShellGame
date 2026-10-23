#!/usr/bin/env bash
# Mock game binary for tests/shell.rs. Every call is logged to $MOCK_LOG as one line of fields joined
# by the unit separator: its kind (`cd-hook`, `guard` or `game`), then the arguments. The MOCK_*
# variables decide what it does: MOCK_EXIT (the status of a game call), MOCK_STDOUT and MOCK_STDERR
# (what it prints), MOCK_PRE and MOCK_POST (the statuses of the `cd` hook; MOCK_NOTE is what `pre`
# prints) and MOCK_GUARD.
US=$'\x1f'
case "${1-}" in cd-hook | guard) kind=$1 ;; *) kind=game ;; esac
entry=$kind
for arg in "$@"; do entry="$entry$US$arg"; done
printf '%s\n' "$entry" >>"$MOCK_LOG"
case "$kind" in
    cd-hook)
        [ "${2-}" = pre ] || exit "${MOCK_POST-0}"
        [ -z "${MOCK_NOTE-}" ] || echo "$MOCK_NOTE" >&2
        exit "${MOCK_PRE-0}"
        ;;
    guard) exit "${MOCK_GUARD-0}" ;;
esac
printf '%s' "${MOCK_STDOUT-}"
printf '%s' "${MOCK_STDERR-}" >&2
exit "${MOCK_EXIT-0}"
