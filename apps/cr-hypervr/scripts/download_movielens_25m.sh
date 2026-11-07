#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "$0")"/.. && pwd)
mkdir -p "$PROJECT_ROOT/data/movielens"

cd "$PROJECT_ROOT/data/movielens"
echo "Downloading MovieLens 25M..."
curl -fL https://files.grouplens.org/datasets/movielens/ml-25m.zip -o ml-25m.zip
echo "Extracting..."
unzip -o ml-25m.zip
echo "MovieLens ready in data/movielens/ml-25m/"

