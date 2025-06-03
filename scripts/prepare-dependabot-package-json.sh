#!/usr/bin/env bash

set -euo pipefail

jq 'delpaths([["scripts"], ["dependencies", "scamplers-core"]])' typescript/scamplers-frontend/package.json > .github/npm-dependencies/package.json
