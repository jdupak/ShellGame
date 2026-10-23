# ShellGame integration for fish: `fish --no-config --init-command '...; source $SHELLGAME_INIT'`; the
# contract is in src/shell/mod.rs. Must stay fish 3.3 compatible: nothing from fish 3.4 or 4 (`$(...)`).

function __shellgame_decode
    printf '%s' "$argv[1]" | command base64 --decode
end

# fish's own `cd` keeps the directory history (`cd -`, prevd, nextd). Keep a copy before the hook
# below replaces it; teleports and the hook call the copy. The guard is for a second `source`.
if not functions -q __shellgame_cd
    functions --copy cd __shellgame_cd
end

# Runs one directive: <stderr file> <version> <verb> [<base64 argument>...]. A closed set, never eval.
function __shellgame_eval
    set -l stderr_file $argv[1]
    set -e argv[1]
    switch "$argv[1] $argv[2] "(count $argv)
        case 'v1 cd 3'
            set -l destination (__shellgame_decode "$argv[3]"); or return 2
            __shellgame_cd -- "$destination"
        case 'v1 export 4'
            set -l name (__shellgame_decode "$argv[3]"); or return 2
            set -l value (__shellgame_decode "$argv[4]"); or return 2
            string match -rq '^[A-Za-z_][A-Za-z0-9_]*$' -- "$name"; or return 2
            set -gx $name "$value"
        case 'v1 exit 2'
            command rm -f $stderr_file
            exit 0
        case '*'
            return 2
    end
end

# Records that the player used `pwd`, for the levels that ask for it. The workspace is only ever
# supplied by the game; never guess it.
function pwd --description 'Print working directory'
    builtin pwd $argv
    set -l rc $status
    test $rc -eq 0; and test -d "$SHELLGAME_WORKSPACE"; and command touch "$SHELLGAME_WORKSPACE/.pwd_used"
    return $rc
end

# The one `cd` hook for every level: the binary decides. Status 3 blocks; any other status, a crash
# or a missing binary allows the move, so a broken hook can never lock the player in.
function cd --description 'Change directory'
    set -l before $PWD
    # fish never sets OLDPWD: `cd -` goes to the newest entry of dirprev, or of dirnext right after a
    # `cd -` (the history toggles direction). Empty when unknown.
    set -l prev ''
    if test "$__fish_cd_direction" = next
        set -q dirnext[-1]; and set prev $dirnext[-1]
    else
        set -q dirprev[-1]; and set prev $dirprev[-1]
    end
    command "$SHELLGAME_BIN" cd-hook pre "$before" "$prev" -- $argv
    test $status -eq 3; and return 1

    __shellgame_cd $argv
    set -l cd_status $status
    test $cd_status -eq 0; and command "$SHELLGAME_BIN" cd-hook post "$before" "$PWD" -- $argv >/dev/null 2>&1
    return $cd_status
end

# A typed chmod outside /tmp is refused (status 3); anything else, also a broken binary, runs it.
function chmod --wraps chmod --description 'Change file modes'
    command "$SHELLGAME_BIN" guard -- $argv
    test $status -eq 3; and return 1
    command chmod $argv
end

# Runs the binary with stderr captured, then replays it: directive lines are executed, the rest is
# passed on unchanged.
function shellgame
    set -l stderr_file (command mktemp); or return 1

    command "$SHELLGAME_BIN" $argv 2>$stderr_file
    set -l exit_code $status

    while read -l line
        set line (string replace -r '\r$' '' -- "$line") # exactly one CR; stderr is otherwise replayed as it is
        if string match -rq '^__SHELLGAME_EXEC__' -- "$line"
            # Split on single spaces, never evaluate.
            set -l words (string split ' ' -- (string replace -r '^__SHELLGAME_EXEC__' '' -- "$line"))
            __shellgame_eval "$stderr_file" $words
        else
            printf '%s\n' "$line" >&2
        end
    end <$stderr_file

    command rm -f $stderr_file
    return $exit_code
end

complete -c shellgame -f
complete -c shellgame -n __fish_use_subcommand -a 'hint submit status reset show levels resume remove exit update'
complete -c shellgame -s h -l help -d 'Zobrazit nápovědu'
complete -c shellgame -n '__fish_seen_subcommand_from submit' -F

# Autostart: last, so that everything above is defined.
if not set -q SHELLGAME_AUTO_STARTED
    set -gx SHELLGAME_AUTO_STARTED 1
    shellgame
end
