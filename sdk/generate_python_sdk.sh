#!/usr/bin/env bash

set -e

PACKAGE_VERSION="0.1.0"

npx @openapitools/openapi-generator-cli generate -i "../kraken_frontend/openapi.json" \
  -g "python" \
  -o "./python-sdk/" \
  --git-repo-id="kraken-project" \
  --git-user-id="myOmikron" \
  --additional-properties="library=asyncio,projectName=kraken-sdk,packageName=kraken_sdk,packageVersion=$PACKAGE_VERSION"
