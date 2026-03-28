#!/bin/bash

# This script builds florca images and release assets, and publishes them on GitHub.
#
# Special requirements:
#
# - [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html) for building the documentation
# - [gh](https://cli.github.com/), authenticated with a GitHub account via `gh auth login`
#
# The script assumes that docker can be run without `sudo`.

set -Eeuo pipefail

version=0.8.0 # Update for each release

read -rp "Enter your GitHub username: " username
read -srp "Enter your GitHub personal access token (classic) with 'write:packages' and 'delete:packages' scopes: " personal_access_token
echo

architecture() {
  case $(uname -m) in
  x86_64) echo "amd64" ;;
  aarch64) echo "arm64" ;;
  *)
    echo "Unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
  esac
}

os() {
  case $(uname -s) in
  Linux) echo "linux" ;;
  Darwin) echo "darwin" ;;
  *)
    echo "Unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
  esac
}

architecture=$(architecture)
os=$(os)
asset_filename=florca-${version}-${os}-${architecture}.tar.gz
book_asset_filename=florca-${version}-book.tar.gz

read -rp "Press Enter to continue with building ${asset_filename} and related images"

# init
rm -rf "dist/build"
mkdir -p "dist/build/florca"

# book
mdbook build book
cp -r book/book dist/build/florca

# cli
docker build -f crates/cli/Dockerfile --output dist/build/florca .

# vendor
mkdir -p "dist/build/florca/vendor"
cp -r packages/fn dist/build/florca/vendor

# examples
cp -r examples dist/build/florca

# config
cp compose.yaml dist/src/.env dist/src/Caddyfile dist/src/deno.json dist/build/florca
cp -r docker-entrypoint-initdb.d dist/build/florca/docker-entrypoint-initdb.d

cd dist/build

# tar
tar -czf "${asset_filename}" florca
tar -czf "${book_asset_filename}" -C florca book

# images
docker compose build

# login
echo "Logging in to GitHub's Container Registry ..."
echo "${personal_access_token}" | docker login ghcr.io --username "${username}" --password-stdin

# publish-images
read -rp "Press Enter to continue with publishing images"
docker push "ghcr.io/floating-orca/deployer:${version}"
docker push "ghcr.io/floating-orca/engine:${version}"
docker push "ghcr.io/floating-orca/deployer:latest"
docker push "ghcr.io/floating-orca/engine:latest"

# publish
read -rp "Press Enter to continue with publishing the assets"
gh release create "v${version}" "${asset_filename}" --title "v${version}" --notes "Release v${version}"

# publish book
gh release upload "v${version}" "${book_asset_filename}"
