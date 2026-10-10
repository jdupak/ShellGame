#!/usr/bin/env bash
# Installs the latest ShellGame release for this machine (Linux or macOS, x86_64 or aarch64).
#
#   curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | bash
#
# Environment overrides:
#   INSTALL_DIR  directory to install `shellgame` into (default: the current directory)
#   REPO         GitHub repository to download from (default: jdupak/ShellGame)
#
# Keep this script compatible with bash 3.2, the system bash on macOS.
set -euo pipefail

REPO="${REPO:-jdupak/ShellGame}"
INSTALL_DIR="${INSTALL_DIR:-${PWD}}"

for tool in curl tar; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "'${tool}' is required but was not found in PATH. Install it and run this script again." >&2
    exit 1
  fi
done

case "$(uname -s)" in
  Linux)
    os_name="linux"
    ;;
  Darwin)
    os_name="macos"
    ;;
  *)
    echo "Unsupported OS: $(uname -s) (supported: Linux, macOS)" >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64|amd64)
    arch_name="x86_64"
    ;;
  aarch64|arm64)
    arch_name="aarch64"
    ;;
  *)
    echo "Unsupported architecture: $(uname -m) (supported: x86_64, aarch64/arm64)" >&2
    exit 1
    ;;
esac

# A shell running under Rosetta 2 reports x86_64 even on Apple Silicon. Prefer the native build.
if [[ "${os_name}" == "macos" && "${arch_name}" == "x86_64" ]] \
  && [[ "$(sysctl -n hw.optional.arm64 2>/dev/null || true)" == "1" ]]; then
  arch_name="aarch64"
fi

asset_name="shellgame-${os_name}-${arch_name}.tar.gz"
release_url="https://github.com/${REPO}/releases/latest/download/${asset_name}"

mkdir -p "${INSTALL_DIR}"
INSTALL_DIR="$(CDPATH='' cd -- "${INSTALL_DIR}" && pwd)"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

echo "Downloading ${release_url}"
if ! curl -fsSL "${release_url}" -o "${tmp_dir}/${asset_name}"; then
  echo "Download failed: ${release_url}" >&2
  echo "Check your connection. A build for ${os_name}/${arch_name} may also not be published yet." >&2
  exit 1
fi

tar -xzf "${tmp_dir}/${asset_name}" -C "${tmp_dir}"
install -m 755 "${tmp_dir}/shellgame" "${INSTALL_DIR}/shellgame"

# Name the command that starts this copy: by name if INSTALL_DIR is on PATH, otherwise by path.
if [[ "$(command -v shellgame || true)" == "${INSTALL_DIR}/shellgame" ]]; then
  run_cmd="shellgame"
elif [[ "${INSTALL_DIR}" == "${PWD}" ]]; then
  run_cmd="./shellgame"
else
  run_cmd="${INSTALL_DIR}/shellgame"
fi

echo "Installed shellgame to ${INSTALL_DIR}/shellgame"
echo "Start the game with:  ${run_cmd}"
echo "Update it later with: ${run_cmd} update"
