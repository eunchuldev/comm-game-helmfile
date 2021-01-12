#!/bin/bash
set -e

ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../" >/dev/null 2>&1 && pwd )"
CST="$ROOT/tests/cst"
CST_BIN="$ROOT/tests/cst/container-structure-test-linux-amd64"

for dir in "$ROOT"/images/*
do
  dir=${dir%*/} 
  name=${dir##*/}
  name=$(echo "$name" | sed 's/^[0-9_\-]*//g')
  if [ "$name" == "$1" ]; then
    echo "build image $name.."
    DOCKER_BUILDKIT=1 docker build -q -t "$name" "$dir"
    echo "test image $name.."
    if [ -f "$CST/$name.yaml" ]; then 
      $CST_BIN test --image "$name" --config "$CST/$name.yaml"
    elif [ -f "$CST/$name.yml" ]; then
      $CST_BIN test --image "$name" --config "$CST/$name.yml"
    else
      echo "no test found for $name image."
    fi
  fi
done
