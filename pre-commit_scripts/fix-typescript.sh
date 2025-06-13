#!/usr/bin/env bash

set -euo pipefail

cd typescript/scamplers-frontend
npm run check
npm run format
npm run lint
