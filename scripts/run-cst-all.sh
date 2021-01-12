#!/bin/bash
set -e

ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../" >/dev/null 2>&1 && pwd )"

echo ------------------------------------------------------------------------
echo 1. Setup test infra

echo start test mysql server
"$ROOT"/tests/mysql/stop.sh
"$ROOT"/tests/mysql/start.sh

echo ------------------------------------------------------------------------

echo ------------------------------------------------------------------------
echo 2. Build image and run Container Structure Test
echo 

CST="$ROOT/tests/cst"
CST_BIN="$ROOT/tests/cst/container-structure-test-linux-amd64"

RESULTS=''
for dir in "$ROOT"/images/*
do
  dir=${dir%*/} 
  name=${dir##*/}
  name=$(echo "$name" | sed 's/^[0-9_\-]*//g')
  echo "build image $name.."
  #image_id=$(DOCKER_BUILDKIT=1 docker build -q -t "$name" "$dir")
  image_size=$(docker images "$name" --format "{{.Size}}" | head -n 1)
  RESULTS="$RESULTS\n$(printf "%-10s %5s" "$name" "$image_size")"
  echo "test image $name.."
  if [ -f "$CST/$name.yaml" ]; then 
    $CST_BIN test --image "$name" --config "$CST/$name.yaml"
  elif [ -f "$CST/$name.yml" ]; then
    $CST_BIN test --image "$name" --config "$CST/$name.yml"
  else
    echo "no test found for $name image."
  fi
  echo
done

echo
echo Built Images
echo -e "$RESULTS"
echo 
echo ------------------------------------------------------------------------
