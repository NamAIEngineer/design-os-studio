#!/usr/bin/env bash
# Print true only when a stable release tag is at least as new as every
# already-published stable release. Used by the release workflow before
# creating a GitHub Release, so an older backfill cannot become Latest.
set -euo pipefail

current="${1:-${GITHUB_REF_NAME#v}}"
current="${current#v}"

if [[ ! "$current" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$current" != *-* ]]; then
  echo "invalid stable release version: $current" >&2
  exit 2
fi

# GitHub marks pre-releases separately; they must never become Latest.
if [[ "$current" == *-* ]]; then
  printf 'false\n'
  exit 0
fi

repo="${GH_REPO:-${GITHUB_REPOSITORY:?GITHUB_REPOSITORY or GH_REPO is required}}"
tags="${RELEASE_TAGS-}"
if [[ -z "$tags" ]]; then
  tags="$(gh release list --repo "$repo" --exclude-drafts --exclude-pre-releases --limit 1000 --json tagName --jq '.[].tagName')"
fi

latest="$({
  printf '%s\n' "$tags" |
    sed -E 's/^v//' |
    grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' || true
} | sort -V | tail -1)"

if [[ -z "$latest" ]] || [[ "$(printf '%s\n' "$latest" "$current" | sort -V | tail -1)" == "$current" ]]; then
  printf 'true\n'
else
  printf 'false\n'
fi
