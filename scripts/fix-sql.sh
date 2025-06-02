#!/usr/bin/env bash

set -euo pipefail

cd db
sqlfluff fix
