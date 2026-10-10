# ShellGame shell integration for Fish
#
# The launcher (src/shell/launch.rs) starts `fish --init-command 'function fish_greeting; end;
# source $SHELLGAME_INIT'` and hands over everything this script needs through the environment,
# so no path is ever pasted into this text:
#   SHELLGAME_WRAPPER   1: the binary knows it runs inside the game shell
#   SHELLGAME_BIN       absolute path of the shellgame binary
#   SHELLGAME_INIT      path of this script
#   SHELLGAME_FD_HOOK   path of the fd-hook launcher (level scripts execute it)
# The binary itself exports SHELLGAME_WORKSPACE and SHELLGAME_LEVEL through the protocol below.

@DEV_SHORTCUTS@

# Decode one base64-encoded protocol argument.
function __shellgame_decode
    printf '%s' "$argv[1]" | command base64 --decode
end

# Keep a copy of fish's own `cd` (directory history, `cd -`, prevd/nextd) before the hook below
# replaces it. Teleports and the hook call the copy, so the history keeps working.
# The guard matters when this script is sourced twice: `cd` would then be our own hook.
if not functions -q __shellgame_cd
    functions --copy cd __shellgame_cd
end

# Execute one closed protocol operation.
# Arguments: <stderr file> <version> <verb> [<base64 argument>...]
# Uses the stock `cd` copy to bypass the user-facing cd hook when the game teleports.
function __shellgame_eval
    set -l stderr_file "$argv[1]"
    set -e argv[1]

    if test (count $argv) -lt 2; or test "$argv[1]" != v1
        return 2
    end

    set -l verb "$argv[2]"

    if test "$verb" = cd; and test (count $argv) -eq 3
        set -l destination (__shellgame_decode "$argv[3]")
        or return 2
        __shellgame_cd -- "$destination"
    else if test "$verb" = export; and test (count $argv) -eq 4
        set -l name (__shellgame_decode "$argv[3]")
        or return 2
        set -l value (__shellgame_decode "$argv[4]")
        or return 2

        string match -rq '^[A-Za-z_][A-Za-z0-9_]*$' -- "$name"
        or return 2
        set -gx $name "$value"
    else if test "$verb" = echo; and test (count $argv) -eq 3
        __shellgame_decode "$argv[3]"
        printf '\n'
    else if test "$verb" = pwd; and test (count $argv) -eq 2
        builtin pwd
    else if test "$verb" = exit; and test (count $argv) -eq 2
        command rm -f "$stderr_file"
        exit 0
    else
        return 2
    end
end

function pwd --description 'Print working directory'
    builtin pwd $argv
    set -l rc $status
    # Record usage for game verification.
    # The workspace is only ever supplied by the game; never guess it.
    if test $rc -eq 0; and test -n "$SHELLGAME_WORKSPACE"; and test -d "$SHELLGAME_WORKSPACE"
        command touch "$SHELLGAME_WORKSPACE/.pwd_used"
    end
    return $rc
end

# The one `cd` hook for every level. The binary decides from its saved state whether a move is
# allowed, so this function never changes between levels.
function cd --description 'Change directory'
    set -l before $PWD

    # fish never sets OLDPWD. The directory `cd -` would move to is the newest entry of dirprev,
    # or of dirnext right after a `cd -` (fish's history toggles direction). Empty when unknown.
    set -l prev ''
    if test "$__fish_cd_direction" = next
        if set -q dirnext[-1]
            set prev $dirnext[-1]
        end
    else if set -q dirprev[-1]
        set prev $dirprev[-1]
    end

    # Ask the binary whether this move is allowed: `cd-hook pre <pwd> <previous dir> -- <cd args>`.
    # It runs directly, not through `shellgame`, so its explanation reaches the terminal live and
    # nothing is relocated or replayed. Exit status 3 means "blocked"; every other status
    # (0, a crash, a missing binary) allows the move, so a broken hook can never lock the player in.
    command "$SHELLGAME_BIN" cd-hook pre "$before" "$prev" -- $argv
    if test $status -eq 3
        return 1
    end

    __shellgame_cd $argv
    set -l cd_status $status

    # Report the completed move so the level can track progress. `$PWD` is where we really are.
    if test $cd_status -eq 0
        command "$SHELLGAME_BIN" cd-hook post "$before" "$PWD" -- $argv >/dev/null 2>&1
    end

    return $cd_status
end

function shellgame
    # Capture stderr in a temporary file so protocol lines can be separated from normal output.
    set -l stderr_file (command mktemp)
    or return 1

    # If the current working directory was deleted, relocate before running the binary.
    if not test -d "$PWD"
        if test -n "$SHELLGAME_WORKSPACE"; and test -d "$SHELLGAME_WORKSPACE"
            builtin cd -- "$SHELLGAME_WORKSPACE" 2>/dev/null; or builtin cd /tmp 2>/dev/null
        else
            builtin cd /tmp 2>/dev/null
        end
    end

    # stdin and stdout stay on the terminal; only stderr is captured and replayed below.
    # The binary is executed as a command with an argv list, never via eval/string splitting.
    command "$SHELLGAME_BIN" $argv 2>$stderr_file
    set -l exit_code $status

    while read -l line
        # Keep exact output except for removing a single trailing CR.
        # Do NOT trim generally; it can corrupt legitimate stderr.
        set line (string replace -r '\r$' '' -- "$line")

        # Anchored minimal protocol: only lines starting exactly with __SHELLGAME_EXEC__ are
        # directives. The payload is split on single spaces and never evaluated.
        if string match -rq '^__SHELLGAME_EXEC__' -- "$line"
            set -l payload (string replace -r '^__SHELLGAME_EXEC__' '' -- "$line")
            set -l words (string split ' ' -- "$payload")
            __shellgame_eval "$stderr_file" $words
        else
            printf '%s\n' "$line" >&2
        end
    end <$stderr_file

    command rm -f "$stderr_file"

    # The binary asks for the game shell to close with the `exit` directive (for example after
    # `shellgame remove`), so no command name is special-cased here.
    return $exit_code
end

# Autocomplete for shellgame
function __fish_shellgame_needs_command
    set cmd (commandline -opc)
    if test (count $cmd) -eq 1
        return 0
    end
    return 1
end

# Disable general file completion by default
complete -c shellgame -f

# Root commands
complete -c shellgame -n "__fish_shellgame_needs_command" -a "init hint submit skip status reset repeat show remove exit update"
complete -c shellgame -s h -l help -d "Zobrazit nápovědu"

# Options for hint
complete -c shellgame -n "__fish_seen_subcommand_from hint" -s r -l repeat -d "Zobrazit znovu všechny již zobrazené nápovědy"
complete -c shellgame -n "__fish_seen_subcommand_from hint" -s h -l help -d "Zobrazit nápovědu"

# Options for repeat
complete -c shellgame -n "__fish_seen_subcommand_from repeat" -l section -d "Číslo sekce ke zopakování"
complete -c shellgame -n "__fish_seen_subcommand_from repeat" -l level -d "ID levelu ke zopakování"
complete -c shellgame -n "__fish_seen_subcommand_from repeat" -s h -l help -d "Zobrazit nápovědu"

# Options for show
complete -c shellgame -n "__fish_seen_subcommand_from show" -l section -d "Zobrazit úvod aktuální sekce"
complete -c shellgame -n "__fish_seen_subcommand_from show" -l level -d "Zobrazit zadání aktuálního levelu"
complete -c shellgame -n "__fish_seen_subcommand_from show" -s h -l help -d "Zobrazit nápovědu"

# Subcommands for dev
complete -c shellgame -n "__fish_seen_subcommand_from dev" -a "jump next prev reload start"

# Allow file completion for submit
complete -c shellgame -n "__fish_seen_subcommand_from submit" -F

# Wrap completions for common aliases
complete -c sg -w shellgame
complete -c g -w shellgame

# Autostart block (runs once when integration is sourced).
# This must be at the end so the shellgame function is already defined.
if not set -q SHELLGAME_AUTO_STARTED
    set -gx SHELLGAME_AUTO_STARTED 1
    shellgame
end
