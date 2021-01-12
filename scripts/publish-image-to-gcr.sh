#!/bin/bash

set -e

prefix="sa-kube"
IMAGE="$1"
TAG="${2:-$NAMESPACE}"

if [ -z "$IMAGE" ]; then
  echo "image-name is missing."
  echo "hint: $0 <image-name> <tag>"
  exit 1
fi
if [ -z "$TAG" ]; then
  echo "tag of image is missing."
  echo "hint: $0 <image-name> <tag>"
  exit 1
fi

ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../" >/dev/null 2>&1 && pwd )"

eval "$(aws ecr get-login --no-include-email)"

for dir in "$ROOT"/images/*
do
  dir=${dir%*/} 
  name=${dir##*/}
  name=$(echo "$name" | sed 's/^[0-9_\-]*//g')
  DOCKER_BUILDKIT=1 docker build -t "$name:$TAG" "$dir"
  if [ "$IMAGE" == "$name" ]; then 
    break
  fi
done

RESULT=''
for dir in "$ROOT"/images/*
do
  dir=${dir%*/} 
  name=${dir##*/}
  name=$(echo "$name" | sed 's/^[0-9_\-]*//g')
  echo $name
  if [ "$IMAGE" == "$name" ]; then 
    rep="${prefix}/${name}"
    aws ecr create-repository --repository-name "$rep" || true
    url=$(aws ecr describe-repositories --repository-names "$rep" --query repositories[0].repositoryUri | tr -d \")
    DOCKER_BUILDKIT=1 docker tag "$name:$TAG" "$url:$TAG"
    DOCKER_BUILDKIT=1 docker push "$url:$TAG"
    RESULT="$RESULT
$url"
  fi
done

echo "The updated repositories:"
echo "$RESULT"
