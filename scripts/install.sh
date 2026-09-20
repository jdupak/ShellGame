#!/usr/bin/env bash
set -euo pipefail

REPO="${REPO:-jdupak/ShellGame}"
INSTALL_DIR="${INSTALL_DIR:-$PWD}"

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "Unsupported OS: $(uname -s) (only Linux is currently published)" >&2
  exit 1
fi
os_name="linux"

case "$(uname -m)" in
  x86_64|amd64)
    arch_name="x86_64"
    ;;
  *)
    echo "Unsupported architecture: $(uname -m) (only x86_64 is currently published)" >&2
    exit 1
    ;;
esac

asset_name="shellgame-${os_name}-${arch_name}.tar.gz"
release_url="https://github.com/${REPO}/releases/latest/download/${asset_name}"

mkdir -p "${INSTALL_DIR}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

curl -fsSL "${release_url}" -o "${tmp_dir}/${asset_name}"
tar -xzf "${tmp_dir}/${asset_name}" -C "${tmp_dir}"
install -m 755 "${tmp_dir}/shellgame" "${INSTALL_DIR}/shellgame"

echo "Installed shellgame to ${INSTALL_DIR}/shellgame"
