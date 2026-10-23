#!/bin/sh
# Installs the latest ShellGame release for this machine (Linux or macOS, x86_64 or aarch64).
#
#   curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | bash
#
# Environment overrides:
#   SHELLGAME_INSTALL_DIR  directory to install `shellgame` into (default: the current directory)
#   SHELLGAME_REPO         GitHub repository to download from (default: jdupak/ShellGame)
#
# Runs in POSIX sh (dash, for `| sh`) and bash 3.2, the system bash on macOS. Everything happens in
# functions and `main` runs on the last line, so a download cut short runs nothing.
set -eu

fail() {
  echo "$*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 ||
    fail "'$1' is required but was not found in PATH. Install it and run this script again."
}

# shellgame-<os>-<arch>.tar.gz, the asset name CI publishes and `shellgame update` downloads.
asset_name() {
  case "$(uname -s)" in
    Linux) os_name=linux ;;
    Darwin) os_name=macos ;;
    *) fail "Unsupported OS: $(uname -s) (supported: Linux, macOS)" ;;
  esac
  case "$(uname -m)" in
    x86_64 | amd64) arch_name=x86_64 ;;
    aarch64 | arm64) arch_name=aarch64 ;;
    *) fail "Unsupported architecture: $(uname -m) (supported: x86_64, aarch64/arm64)" ;;
  esac
  # A shell running under Rosetta 2 reports x86_64 even on Apple Silicon. Prefer the native build.
  if [ "$os_name" = macos ] && [ "$arch_name" = x86_64 ] &&
    [ "$(sysctl -n hw.optional.arm64 2>/dev/null || true)" = 1 ]; then
    arch_name=aarch64
  fi
  echo "shellgame-$os_name-$arch_name.tar.gz"
}

main() {
  need curl
  need tar
  repo="${SHELLGAME_REPO:-jdupak/ShellGame}"
  install_dir="${SHELLGAME_INSTALL_DIR:-$PWD}"
  asset="$(asset_name)"
  url="https://github.com/$repo/releases/latest/download/$asset"

  mkdir -p "$install_dir"
  install_dir="$(CDPATH='' cd -- "$install_dir" && pwd)"

  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT
  trap 'exit 1' HUP INT TERM

  echo "Downloading $url"
  if ! curl --proto '=https' --tlsv1.2 -fsSL --connect-timeout 20 --retry 3 \
    -o "$tmp_dir/$asset" "$url"; then
    echo "Download failed: $url" >&2
    fail "Check your connection. A build of $asset may also not be published yet."
  fi

  tar -xzf "$tmp_dir/$asset" -C "$tmp_dir" shellgame ||
    fail "The downloaded archive $asset is damaged. Run this script again."
  # Copied next to its place, tried there (/tmp may forbid running programs) and renamed over the
  # old copy, which is never half written, not even while it runs.
  new="$install_dir/.shellgame.new"
  cp "$tmp_dir/shellgame" "$new"
  chmod 755 "$new"
  if ! "$new" --version >/dev/null 2>&1; then
    rm -f "$new"
    fail "The downloaded program does not run on this machine ($asset)."
  fi
  mv -f "$new" "$install_dir/shellgame"

  # Name the command that starts this copy: by name if it is the one on PATH, otherwise by path.
  if [ "$(command -v shellgame || true)" = "$install_dir/shellgame" ]; then
    run_cmd=shellgame
  elif [ "$install_dir" = "$PWD" ]; then
    run_cmd=./shellgame
  else
    run_cmd="$install_dir/shellgame"
  fi

  echo "Installed $("$install_dir/shellgame" --version) to $install_dir/shellgame"
  echo "Start the game with:  $run_cmd"
  echo "Update it later with: $run_cmd update"
}

main "$@"
