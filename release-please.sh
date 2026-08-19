#!/usr/bin/env bash
set -e

ARGS=(
  --token="$(gh auth token)"
  --repo-url="google/cddlconv"
  --config-file="release-please-config.json"
  --manifest-file=".release-please-manifest.json"
  "$@"
)

release-please github-release "${ARGS[@]}"
release-please release-pr "${ARGS[@]}"
