#!/usr/bin/env bash
# Release helper for OpenChat.
#
#   scripts/release.sh bump <api|desktop|webui> <patch|minor|major>
#       Bump the component's version (manifest + lockfile), verify it builds,
#       and commit on the current branch. Open a PR with the result.
#
#   scripts/release.sh tag <api|desktop>
#       On an up-to-date, clean main, tag the component's current version and
#       push the tag. The push triggers the component's release workflow.
#
#   scripts/release.sh version <api|desktop|webui>
#       Print the component's current version.
#
# The web UI has no release workflow; deploy it with webui/build-and-pub-to-s3.sh.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

die() { echo "error: $*" >&2; exit 1; }

usage() {
    sed -n '2,15p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    exit 1
}

component_dir() {
    case "$1" in
        api) echo "$ROOT/api" ;;
        desktop) echo "$ROOT/tauri" ;;
        webui) echo "$ROOT/webui" ;;
        *) die "unknown component '$1' (expected api, desktop, or webui)" ;;
    esac
}

tag_prefix() {
    case "$1" in
        api) echo "api-v" ;;
        desktop) echo "desktop-v" ;;
        webui) die "webui has no release tag; deploy it with webui/build-and-pub-to-s3.sh" ;;
        *) die "unknown component '$1'" ;;
    esac
}

current_version() {
    local dir
    dir="$(component_dir "$1")"
    if [ "$1" = "webui" ]; then
        node -p "require('$dir/package.json').version"
    else
        grep -m1 '^version' "$dir/Cargo.toml" | cut -d'"' -f2
    fi
}

next_version() {
    local version="$1" level="$2" major minor patch
    IFS=. read -r major minor patch <<< "$version"
    case "$level" in
        major) echo "$((major + 1)).0.0" ;;
        minor) echo "$major.$((minor + 1)).0" ;;
        patch) echo "$major.$minor.$((patch + 1))" ;;
        *) die "unknown bump level '$level' (expected patch, minor, or major)" ;;
    esac
}

cmd_bump() {
    local component="$1" level="$2" dir old new
    dir="$(component_dir "$component")"
    old="$(current_version "$component")"
    new="$(next_version "$old" "$level")"

    [ -z "$(git -C "$ROOT" status --porcelain)" ] || die "working tree is not clean"
    [ "$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)" != "main" ] || die "bump on a branch, not main (main requires a PR)"

    echo "Bumping $component: $old -> $new"
    if [ "$component" = "webui" ]; then
        (cd "$dir" && npm version "$new" --no-git-tag-version >/dev/null)
        (cd "$dir" && npx tsc --noEmit)
        git -C "$ROOT" add "$dir/package.json" "$dir/package-lock.json"
    else
        sed -i.bak "0,/^version = \"$old\"/s//version = \"$new\"/" "$dir/Cargo.toml"
        rm -f "$dir/Cargo.toml.bak"
        # cargo check refreshes the package's own entry in Cargo.lock
        (cd "$dir" && cargo check --all-targets --quiet)
        git -C "$ROOT" add "$dir/Cargo.toml" "$dir/Cargo.lock"
    fi

    git -C "$ROOT" commit --quiet -m "chore(release): $component $new"
    if [ "$component" = "webui" ]; then
        echo "Committed. Open a PR, merge it, then deploy with webui/build-and-pub-to-s3.sh"
    else
        echo "Committed. Open a PR, merge it, then run: scripts/release.sh tag $component"
    fi
}

cmd_tag() {
    local component="$1" version tag
    tag="$(tag_prefix "$component")"
    version="$(current_version "$component")"
    tag="$tag$version"

    [ "$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)" = "main" ] || die "tags must be cut from main"
    [ -z "$(git -C "$ROOT" status --porcelain)" ] || die "working tree is not clean"
    git -C "$ROOT" fetch --quiet --tags origin main
    [ "$(git -C "$ROOT" rev-parse HEAD)" = "$(git -C "$ROOT" rev-parse origin/main)" ] \
        || die "local main is not in sync with origin/main (git pull first)"
    if git -C "$ROOT" rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
        die "tag $tag already exists (bump the version first)"
    fi

    git -C "$ROOT" tag -a "$tag" -m "OpenChat $component $version"
    git -C "$ROOT" push --quiet origin "$tag"
    echo "Pushed $tag. Watch the release: gh run list --workflow $([ "$component" = api ] && echo api-release.yml || echo tauri-release.yml) --limit 1"
}

[ $# -ge 2 ] || usage
case "$1" in
    bump) [ $# -eq 3 ] || usage; cmd_bump "$2" "$3" ;;
    tag) [ $# -eq 2 ] || usage; cmd_tag "$2" ;;
    version) [ $# -eq 2 ] || usage; current_version "$2" ;;
    *) usage ;;
esac
