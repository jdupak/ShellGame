---
name: playtest
description: Play ShellGame by hand in a throwaway environment (fake HOME, state directory and workspace) in bash and fish, checking intro paging, hints, wrong answers, strict cd levels, resume, levels, reset and the sandbox. Use after changing levels, the UI, the shell scripts, the sandbox or the CLI, before a release, or to see a level the way a student does.
---

# Playtest in a throwaway environment

Never play against your own save or your real `$HOME`: the checks below change and delete files on
purpose. The snippets are bash. Run them from the repository root in an ordinary terminal, not inside a
game shell.

## 1. Build and set up

```bash
cargo build
BIN="$PWD/target/debug/shellgame"
T=$(mktemp -d /var/tmp/shellgame-playtest.XXXXXX)    # outside /tmp: the sandbox must have something to protect
mkdir -p "$T/home" "$T/xdg"
echo precious > "$T/home/precious.txt"
W=/tmp/shellgame-playtest-$$                           # the workspace must be inside /tmp
export T W BIN
run() { env HOME="$T/home" XDG_CONFIG_HOME="$T/xdg" SHELLGAME_WORKSPACE="$W" "$BIN" "$@"; }
```

Do not use `cargo run` with a changed `HOME`: cargo and rustup live there. Progress goes to
`$T/xdg/shellgame/state.json`, level files to `$W`; confirm both after the first screen.
Do not put `$T` under `/tmp`: everything there is writable inside the sandbox, so it would prove nothing.

## 2. Start the game, once per shell

```bash
run --shell bash        # then run --shell fish (keep the state, or start fresh: see step 4)
```

The game needs a terminal. Without one, drive it through tmux on a private socket:

```bash
tmux -L sgplay new-session -d -s sg -x 110 -y 40 \
  "env HOME=$T/home XDG_CONFIG_HOME=$T/xdg SHELLGAME_WORKSPACE=$W $BIN --shell bash"
tmux -L sgplay send-keys -t sg 'shellgame hint' Enter
tmux -L sgplay capture-pane -t sg -p
tmux -L sgplay kill-server
```

Subcommands also run outside the game shell and without a terminal: `run resume 3.2; run status`.

## 3. What to try, in both shells

- **Start.** Intro pages advance with Enter; under each level panel is the isolation line ("Izolace
  zapnuta: ..."). Ctrl-C at an Enter prompt must leave the shell usable.
- **Hints.** `shellgame hint` three times: the hints escalate, and every call shows the unlocked ones
  again with one more, until it says there are none left.
- **Answers.** `shellgame submit nonsense` on a level that takes an answer: a short reply that does not
  reveal the answer. A bare `shellgame submit`: a reply that shows the example command.
- **Strict cd.** In a level that practises one exact `cd` (section 1), a wrong `cd` is blocked with a red
  panel and the `shellgame reset` tip; `cd` back to the start directory works; `shellgame reset` puts you
  in the start directory.
- **Resume.** From fresh state (see step 4): `shellgame resume 3.2` typed in an ordinary shell starts the
  game shell in level 3.2 (also with `--shell fish` and for `reset` and `submit`); inside, `shellgame status`
  and `shellgame exit` print "Na jiném počítači pokračujte příkazem: shellgame resume 3.2", `shellgame resume 3`
  goes to the first level of section 3 after its intro, `resume` alone and `resume 99` fail (status 1) and
  point to `shellgame levels`, which lists every level with "▸ zde" and "✓" (also outside the game shell,
  where `exit` says you are not in it). Resuming a completed level keeps it counted in `status`.
- **Show.** `shellgame show` (the current assignment), `show 1.2` (another one) and `show 1` (the intro
  of a section) print without touching the game; `show 99` and `show x` fail with a message and status 1.
- **Deleted directory.** In a level that is judged on where you stand (`shellgame resume 1.3`) run
  `mkdir x && cd x && rmdir "$PWD"` (fish: `mkdir x; and cd x; and rmdir "$PWD"`), then
  `shellgame submit`: it says the directory is gone, and `shellgame reset` puts you back at the start.
- **Reset after `chmod 000`.** `shellgame resume 8`, run `chmod 000` on the level's files and directories
  in the workspace, then `shellgame reset`: the level is rebuilt and you are back at its start.
- **Sandbox (Linux).** Inside the game shell each of these must fail and leave the file intact:
  `rm "$HOME/precious.txt"`, `echo x > "$HOME/precious.txt"`, `touch "$HOME/new"`,
  `mv "$HOME/precious.txt" "$HOME/moved"`, `mkdir "$HOME/d"`, `chmod 777 "$HOME/precious.txt"` (the typed
  `chmod` is refused with a Czech message, the others with "Permission denied"). These must work: files
  in the workspace and in `/tmp`, and `shellgame submit`. `sudo true` failing is expected, and
  `shellgame update` only tells you to run it outside. Known limits, not bugs: `command chmod`,
  `/usr/bin/chmod`, `find -exec chmod` and `touch` on an existing file still work. Leave the game shell
  and check that `cat "$T/home/precious.txt"` still prints `precious` and `ls -A "$T/home"` lists only
  that file.
- **Opt-out.** `run --unsafe --shell bash` starts with a red warning and no isolation. Run destructive
  commands there only against files under `$T`.
- **Without Landlock** (a VM or container with an old kernel) the game must refuse to start and explain
  why; `--unsafe` must still start it.

Note which shell and which level each problem happened in.

## 4. Start fresh, and clean up

Section 8 leaves mode-000 files and directories, so restore permissions before deleting:

```bash
# a fresh game with the same fake home
chmod -R u+rwx "$T/xdg" "$W" 2>/dev/null; rm -rf "$T/xdg" "$W"; mkdir "$T/xdg"
# when you are done
chmod -R u+rwx "$T" "$W" 2>/dev/null; rm -rf "$T" "$W"
```

Remove only what you created here, and kill the tmux server if you started one
(`tmux -L sgplay kill-server`).
