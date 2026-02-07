#!/bin/sh
set -e

# Build if needed
if [ ! -d "./result/worker" ]; then
  echo "Building Cloudflare Worker..."
  nix build .#worker
fi

echo "Deploying to Cloudflare..."

# Create temp directory for deployment
DEPLOY_DIR=$(mktemp -d)
trap "chmod -R +w $DEPLOY_DIR 2>/dev/null; rm -rf $DEPLOY_DIR" EXIT

# Copy worker files to writable location
cp -r ./result/worker/* $DEPLOY_DIR/
chmod -R +w $DEPLOY_DIR
cd $DEPLOY_DIR

# Deploy and let wrangler/worker-build handle the build
# Use nix-shell to get wrangler and build tools
nix-shell -p wrangler worker-build cargo rustc lld --run "wrangler deploy $*"

echo ""
echo "Deployment complete!"
echo ""
echo "Test endpoint: curl https://cloudfree-mcp.<your-subdomain>.workers.dev/health"
