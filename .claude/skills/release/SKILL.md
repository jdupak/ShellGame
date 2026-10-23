---
name: release
description: Build and publish ShellGame. Explains how CI turns a push to main into a build-N GitHub release, the dry run with workflow_dispatch, the PUBLISH_MACOS hold-back, and how scripts/install.sh and `shellgame update` pick up releases. Use when asked to release or publish, to test the release pipeline, to enable macOS assets, or to change .github/workflows/ci.yml or scripts/install.sh.
---

# Release

A push to `main` publishes to students: `scripts/install.sh` and `shellgame update` serve the latest
release. **Never push, merge or tag unless the maintainer asked for exactly that.** A dry run needs the
branch on the remote, so it needs their go-ahead too.

## How CI works (`.github/workflows/ci.yml`)

| Job | Runs on | Does |
| --- | --- | --- |
| `test` | matrix: ubuntu-24.04 and ubuntu-22.04 | `cargo test --locked` with `SHELLGAME_TEST_LANDLOCK=required` on both legs; 22.04 has the oldest supported fish (3.3) and bash (5.1). Only the 24.04 leg also runs shellcheck of every tracked `*.sh`, `cargo fmt --check`, `cargo doc` (warnings denied) and clippy `-D warnings` |
| `test-macos` | macos-latest | system bash 3.2 first on `PATH`; `cargo test --locked --test shell` blocks (bash 3.2 parity of the integration script), the full `cargo test --locked` is non-blocking |
| `build` | after `test` (both legs) and `test-macos` | on a push to the default branch and on `workflow_dispatch`: one archive per target, each holding exactly one file, `shellgame`; `SHELLGAME_BUILD=build-<run number>` is compiled in and shown by `--version`; the `Smoke test` step runs the built binary and checks that (x86_64 macOS runs under Rosetta, and only prints a notice when it is missing); on Linux, `Smoke test the game shell` checks that the binary is static and that it starts a confined game shell that runs a typed command, saves state and cannot write to the home |
| `publish` | after `build` | push to the default branch only, one run at a time: a run whose commit is no longer the tip of the default branch (overtaken, or started again later) publishes nothing; otherwise GitHub release `build-<run number>` with the archives, marked latest |

Assets are `shellgame-{linux,macos}-{x86_64,aarch64}.tar.gz` (static musl on Linux). `scripts/install.sh`
and `shellgame update` download `releases/latest/download/<asset>`, so the names are an interface: do
not rename them. The publish job tags the built commit itself; do not create tags or releases by hand.
The crate version in `Cargo.toml` is not bumped per release.

## Dry run (builds everything, publishes nothing)

```bash
BRANCH=$(git branch --show-current)
gh workflow run ci.yml --ref "$BRANCH"       # the workflow file must exist on the default branch
RUN=$(gh run list -w ci.yml -b "$BRANCH" -e workflow_dispatch -L 1 --json databaseId --jq '.[0].databaseId')
gh run watch "$RUN" --exit-status
D=$(mktemp -d); gh run download "$RUN" -n shellgame-linux-x86_64.tar.gz -D "$D"
tar -tzf "$D/shellgame-linux-x86_64.tar.gz"  # must list only: shellgame
```

Run `gh workflow run` only after the maintainer pushed the branch. Archives of a dry run are workflow
artifacts; the `publish` job is skipped. In the run, `test` shows two legs, `test (ubuntu-24.04)` and
`test (ubuntu-22.04)` (the 22.04 leg skips the shellcheck, formatting, docs and clippy steps), and the
`Smoke test` step of every `build` leg prints `shellgame <version> (build-<run number>)` (the x86_64
macOS leg may print a no-Rosetta notice instead).

## macOS assets: `PUBLISH_MACOS`

The macOS legs always build (non-blocking), but `publish` deletes `shellgame-macos-*` while the
`PUBLISH_MACOS` variable at the top of `ci.yml` is `"false"`. Flip it to `"true"` only after the macOS
content review (GNU-only spots in the levels: `ls --help`, `wc` padding, `file` wording, bracket globs
under locale collation) has made the macOS replay green. Do these together: make the full macOS `Test`
step blocking (drop `continue-on-error` there and on the macOS `build` legs), flip the variable, and
update the macOS sentence in `README.md`.

## Before a release

1. `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`, then
   `SHELLGAME_TEST_LANDLOCK=required cargo test --locked`.
2. Playtest in bash and fish (the `playtest` skill): resume, strict cd, reset, the sandbox.
3. Hosted runners have recent kernels, so old Landlock ABIs and kernels without Landlock (Debian 11,
   RHEL 8 and 9.0 to 9.5, Ubuntu 20.04 with its original kernel, WSL 1) are checked by hand: the game
   must isolate as far as the kernel allows, or refuse and explain, and `--unsafe` must start it.
4. Ask the maintainer to push (a branch for a dry run, `main` for the release).

## After the publish run

```bash
gh release view                              # the newest release, build-<N>: the Linux archives are attached
curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | SHELLGAME_INSTALL_DIR="$(mktemp -d)" sh
```

The script prints where it installed `shellgame`; `<that path> --version` must report `build-<N>`.
From an older install, `shellgame update` (run outside the game shell) must replace the binary and keep
the save.

## Changing the pipeline or the installer

Keep `scripts/install.sh` POSIX `sh` (students pipe it into `sh` or `bash`; it must also run in dash
and bash 3.2), with its body in functions called from the last line, and clean under `shellcheck`. The `build` job installs
no `musl-tools`: no dependency compiles C (`Cargo.lock` has no `cc` crate), so rustc's bundled musl is
enough; a crate with a C build script would need `musl-tools` in the Linux legs again. Validate edits
locally and prove them with a dry run:

```bash
git ls-files -z '*.sh' | xargs -0 shellcheck   # the "Lint shell scripts" step: every tracked script
uv run --no-project --with pyyaml python -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'
```
