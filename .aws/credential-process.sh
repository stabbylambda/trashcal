#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${OP_CONNECT_HOST:-}" || -z "${OP_CONNECT_TOKEN:-}" ]]; then
  if ! op account get &>/dev/null; then
    eval "$(op signin)" >/dev/tty 2>&1
  fi
fi

op --cache inject --in-file "$DEVENV_ROOT/.aws/credential-process.json"
