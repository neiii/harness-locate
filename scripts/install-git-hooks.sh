#!/bin/sh

set -eu

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "Error: not a git repository" >&2
  exit 1
fi

GIT_DIR="$(git rev-parse --git-dir)"
HOOKS_DIR="$GIT_DIR/hooks"

if [ ! -d .githooks ]; then
  echo "Error: .githooks directory not found" >&2
  exit 1
fi

mkdir -p "$HOOKS_DIR"

install_hook() {
  name="$1"
  src=".githooks/$name"
  dst="$HOOKS_DIR/$name"

  if [ ! -f "$src" ]; then
    echo "Error: missing $src" >&2
    exit 1
  fi

  cp "$src" "$dst"
  chmod +x "$dst"
}

install_hook pre-commit
install_hook post-merge

echo "Installed git hooks into: $HOOKS_DIR"
