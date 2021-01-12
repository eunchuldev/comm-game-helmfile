#!/bin/bash

set -e

PROFILE=$1
REGION=$2

ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../" >/dev/null 2>&1 && pwd )"

eval $(minikube -p minikube docker-env)

for dir in $ROOT/images/*
do
  dir=${dir%*/} 
  name=${dir##*/}
  name=$(echo $name | sed 's/^[0-9_\-]*//g')
  DOCKER_BUILDKIT=1 docker build -t $name $dir
done
