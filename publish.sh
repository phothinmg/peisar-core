#!/usr/bin/env bash
set -e

# Extract package names sorted by dependency topology
packages=$(cargo metadata --format-version 1 | jq -r '.workspace_members[]' | cut -d' ' -f1)

for pkg in $packages; do
    echo "Publishing $pkg..."
    # --no-verify may be needed if local path replacements haven't propagated to crates.io yet
    cargo publish -p "$pkg"
    
    # Optional: sleep for a few seconds to prevent hitting crates.io rate limits
    sleep 5
done
