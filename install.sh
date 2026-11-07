#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
user_home="${HOME:?HOME must be set}"
cargo_root="${CARGO_HOME:-$user_home/.cargo}"
case "$cargo_root" in
  /*) ;;
  *) echo 'CARGO_HOME must be an absolute path' >&2; exit 1 ;;
esac

binary="$cargo_root/bin/taskotto"
if [[ -f "$project_dir/Cargo.toml" ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo 'Rust/Cargo is required to build from source: https://rustup.rs' >&2
    exit 1
  fi
  cargo install --path "$project_dir" --locked --force --root "$cargo_root"
else
  # Release archives contain the prebuilt binary instead of Cargo.toml.
  source_binary="$project_dir/taskotto"
  if [[ ! -f "$source_binary" || ! -x "$source_binary" ]]; then
    echo 'Taskotto release binary is missing or not executable' >&2
    exit 1
  fi
  mkdir -p -- "$(dirname -- "$binary")"
  temporary="$(mktemp "$(dirname -- "$binary")/.taskotto.XXXXXX")"
  trap 'rm -f -- "$temporary"' EXIT
  install -m755 -- "$source_binary" "$temporary"
  mv -f -- "$temporary" "$binary"
  trap - EXIT
fi

"$binary" --install-desktop
echo 'Installed Taskotto and its application-menu launcher and icon.'
