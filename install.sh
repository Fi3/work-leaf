#!/usr/bin/env bash
set -euo pipefail

repo="${WORK_LEAF_INSTALL_REPO:-Fi3/work-leaf}"
install_dir="${WORK_LEAF_INSTALL_DIR:-/usr/local/bin}"
binary_name="work-leaf"

usage() {
  cat <<EOF
Usage: install.sh

Installs the latest Work Leaf release for Linux or macOS.

Environment:
  WORK_LEAF_INSTALL_DIR       install directory, default: /usr/local/bin
  WORK_LEAF_INSTALL_VERSION   release tag to install, for example: v0.1.1
  WORK_LEAF_INSTALL_REPO      GitHub repo, default: Fi3/work-leaf
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required to download Work Leaf releases" >&2
  exit 1
fi

target_from_os() {
  if [[ -n "${WORK_LEAF_INSTALL_TARGET:-}" ]]; then
    printf '%s\n' "$WORK_LEAF_INSTALL_TARGET"
    return
  fi

  local os
  local arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Linux:x86_64|Linux:amd64)
      printf 'x86_64-unknown-linux-gnu\n'
      ;;
    Linux:aarch64|Linux:arm64)
      printf 'aarch64-unknown-linux-gnu\n'
      ;;
    Darwin:x86_64)
      printf 'x86_64-apple-darwin\n'
      ;;
    Darwin:arm64|Darwin:aarch64)
      printf 'aarch64-apple-darwin\n'
      ;;
    *)
      echo "unsupported OS or CPU for install.sh: $os $arch" >&2
      echo "download a matching release archive manually from https://github.com/$repo/releases" >&2
      exit 1
      ;;
  esac
}

latest_release_tag() {
  if [[ -n "${WORK_LEAF_INSTALL_VERSION:-}" ]]; then
    printf '%s\n' "$WORK_LEAF_INSTALL_VERSION"
    return
  fi

  local api_url="https://api.github.com/repos/$repo/releases/latest"
  local tag
  tag="$(
    curl -fsSL "$api_url" |
      sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
      head -n 1
  )"

  if [[ -z "$tag" ]]; then
    echo "could not resolve the latest Work Leaf release tag from $api_url" >&2
    exit 1
  fi

  printf '%s\n' "$tag"
}

installed_version() {
  local installed_bin="$1"

  if [[ ! -x "$installed_bin" ]]; then
    return
  fi

  "$installed_bin" --version 2>/dev/null | awk 'NR == 1 && $1 == "work-leaf" { print $2 }'
}

download_asset() {
  local url="$1"
  local output="$2"

  echo "Downloading $url"
  curl -fL --retry 3 --proto '=https,file' --tlsv1.2 -o "$output" "$url"
}

verify_checksum() {
  local work_dir="$1"
  local checksum_file="$2"

  echo "Verifying checksum"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$work_dir" && sha256sum -c "$checksum_file")
  elif command -v shasum >/dev/null 2>&1; then
    (cd "$work_dir" && shasum -a 256 -c "$checksum_file")
  else
    echo "sha256sum or shasum is required to verify Work Leaf release checksums" >&2
    exit 1
  fi
}

install_binary() {
  local source_bin="$1"
  local installed_bin="$2"
  local sudo_command=()

  if mkdir -p "$install_dir" 2>/dev/null && [[ -w "$install_dir" ]]; then
    :
  elif command -v sudo >/dev/null 2>&1; then
    sudo mkdir -p "$install_dir"
    sudo_command=(sudo)
  else
    echo "$install_dir is not writable; set WORK_LEAF_INSTALL_DIR to a writable directory" >&2
    exit 1
  fi

  if command -v install >/dev/null 2>&1; then
    "${sudo_command[@]}" install -m 0755 "$source_bin" "$installed_bin"
  else
    "${sudo_command[@]}" cp "$source_bin" "$installed_bin"
    "${sudo_command[@]}" chmod 0755 "$installed_bin"
  fi
}

target="$(target_from_os)"
release_tag="$(latest_release_tag)"
release_version="${release_tag#v}"
archive_name="work-leaf-$target.tar.gz"
checksum_name="$archive_name.sha256"
installed_bin="$install_dir/$binary_name"
current_version="$(installed_version "$installed_bin")"

if [[ "$current_version" == "$release_version" ]]; then
  echo "work-leaf $release_tag is already installed at $installed_bin"
  exit 0
fi

if [[ -n "${WORK_LEAF_INSTALL_BASE_URL:-}" ]]; then
  download_base_url="${WORK_LEAF_INSTALL_BASE_URL%/}"
else
  download_base_url="https://github.com/$repo/releases/download/$release_tag"
fi

temp_dir="$(mktemp -d)"
trap 'rm -rf "$temp_dir"' EXIT

download_asset "$download_base_url/$archive_name" "$temp_dir/$archive_name"
download_asset "$download_base_url/$checksum_name" "$temp_dir/$checksum_name"
verify_checksum "$temp_dir" "$checksum_name"

tar -xzf "$temp_dir/$archive_name" -C "$temp_dir"
extracted_bin="$temp_dir/work-leaf-$target/$binary_name"

if [[ ! -f "$extracted_bin" ]]; then
  echo "release archive did not contain $binary_name at work-leaf-$target/$binary_name" >&2
  exit 1
fi

chmod 0755 "$extracted_bin"
install_binary "$extracted_bin" "$installed_bin"

echo "Installed work-leaf $release_tag to $installed_bin"
