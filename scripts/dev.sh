#!/bin/sh
set -e

# Build if needed
if [ ! -d "./result/worker" ]; then
  echo "Building Cloudflare Worker..."
  nix build .#worker
fi

echo "Starting dev server..."

# Create temp directory for dev
DEV_DIR=$(mktemp -d)
trap "chmod -R +w $DEV_DIR 2>/dev/null; rm -rf $DEV_DIR" EXIT

# Copy worker files to writable location
cp -r ./result/worker/* $DEV_DIR/
chmod -R +w $DEV_DIR
cd $DEV_DIR

# Run dev server
# Use nix-shell to get wrangler and build tools
nix-shell -p wrangler worker-build cargo rustc lld --run "wrangler dev $*"
