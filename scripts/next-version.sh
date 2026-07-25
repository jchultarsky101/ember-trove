#!/usr/bin/env bash
# Computes the next semver based on conventional commits since the last tag.
# Usage: ./scripts/next-version.sh [major|minor|patch]
#   - With argument: bumps that component unconditionally
#   - Without argument: auto-detects from commit messages
#
# Conventional commit prefixes:
#   feat!: or BREAKING CHANGE: → major
#   feat:                      → minor
#   fix: / chore: / refactor:  → patch

REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

CURRENT=$(grep '^version' api/Cargo.toml | head -1 | grep -o '[0-9]*\.[0-9]*\.[0-9]*')
if [ -z "$CURRENT" ]; then
  echo "Error: could not read version from api/Cargo.toml" >&2
  exit 1
fi

IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

if [ -n "$1" ]; then
  BUMP="$1"
else
  # Auto-detect from commits since the last release tag.
  # NOT `git describe`: release tags live on main-side merge commits that are
  # never ancestors of develop (Git Flow), so describe walks back to whatever
  # ancient tag happens to be reachable and scans several releases of commits
  # (found 2026-07-25: it reached v2.22.2 and proposed minor for a deps-only
  # release). `A..HEAD` works fine even when A is not an ancestor.
  LAST_TAG=$(git tag --list 'v*' --sort=-v:refname | head -1)
  if [ -z "$LAST_TAG" ]; then
    COMMITS=$(git log --oneline 2>/dev/null)
  else
    COMMITS=$(git log "${LAST_TAG}..HEAD" --oneline 2>/dev/null)
  fi

  if echo "$COMMITS" | grep -qE '^[a-f0-9]+ (feat!|.*BREAKING CHANGE)'; then
    BUMP="major"
  elif echo "$COMMITS" | grep -qE '^[a-f0-9]+ feat(\([^)]+\))?:'; then
    BUMP="minor"
  else
    BUMP="patch"
  fi
fi

case "$BUMP" in
  major) NEXT="$((MAJOR+1)).0.0" ;;
  minor) NEXT="${MAJOR}.$((MINOR+1)).0" ;;
  patch) NEXT="${MAJOR}.${MINOR}.$((PATCH+1))" ;;
  *)
    echo "Error: unknown bump type '$BUMP'. Use major|minor|patch." >&2
    exit 1
    ;;
esac

echo "$NEXT"
