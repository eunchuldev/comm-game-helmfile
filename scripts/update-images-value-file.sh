#!/bin/bash

set -e

IMAGE="$1"
ENV="$HELMFILE_ENVIRONMENT"
TAG="$(git rev-parse --short HEAD)"

if [ -z "$IMAGE" ]; then
  echo "image-name is missing."
  echo "build and push all images.."
  echo "hint: $0 <image-name> <tag>"
fi
if [ -z "$TAG" ]; then
  echo "tag of image is missing."
  echo "hint: $0 <image-name> <tag>"
  exit 1
fi

ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../" >/dev/null 2>&1 && pwd )"
IMAGE_VALUES_YAML="$ROOT/envs/${ENV}/images.yaml"

for dir in "$ROOT"/images/*
do
  dir="${dir%*/}"
  name="${dir##*/}"
  prefix="$(echo "$name" | grep -o '^[0-9]*')"
  name="$(echo "$name" | sed 's/^[0-9_\-]*//g')"

  if [ "$IMAGE" == "$name" ]; then 
    target_prefix="$prefix"
    break
  fi
done

for dir in "$ROOT"/images/*
do
  dir="${dir%*/}"
  name="${dir##*/}"
  prefix="$(echo "$name" | grep -o '^[0-9]*')"
  name="$(echo "$name" | sed 's/^[0-9_\-]*//g')"

  if [ -n "$target_prefix" ] && [ "$prefix" == "$target_prefix" ]; then
    DOCKER_BUILDKIT=1 docker build -t "$name" "$dir"
  fi
  if [ "$IMAGE" == "$name" ]; then 
    DOCKER_BUILDKIT=1 docker build -t "$name" "$dir"
    break
  fi
done

for dir in "$ROOT"/images/*
do
  dir=${dir%*/} 
  name=${dir##*/}
  name=$(echo "$name" | sed 's/^[0-9_\-]*//g')
  echo $name
  if [ -z "$IMAGE" ] || [ "$IMAGE" == "$name" ]; then 
    url="gcr.io/${GCP_PROJECT}/${name}"
    DOCKER_BUILDKIT=1 docker tag "$name" "$url:$TAG"
    DOCKER_BUILDKIT=1 docker push "$url:$TAG"
    if [ -n "$url" ] && [ -n "$TAG" ]; then
      sed -i "/^\([[:space:]]*${name}: \).*/s//\1${url//\//\\/}:${TAG}/" $IMAGE_VALUES_YAML
    fi
    echo "The updated repositories:"
    echo "$url:$TAG"
    if [ "$IMAGE" == "$name" ]; then break; fi
  fi
done
