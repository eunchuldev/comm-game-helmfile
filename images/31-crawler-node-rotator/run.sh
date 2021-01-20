#!/bin/sh
set -e

max_node_count="${MAX_NODE_COUNT:-1000}"

node_lifetime_min="${NODE_LIFETIME_MIN:-60}"

pod_label="${POD_LABEL:-app=dc-crawler-worker}"

node_selector="${NODE_SELECTOR:-cloud.google.com/gke-nodepool=crawler-pool}"

min_node_creation_time="$(date --date="-${node_lifetime_min}min" -u +"%Y-%m-%dT%H:%M:%SZ")"

nodes="$(kubectl get node \
  -l $node_selector \
  --sort-by=.metadata.creationTimestamp \
  -o=jsonpath="{range .items[?(@.metadata.creationTimestamp < '${min_node_creation_time}')]}{.metadata.name}{'\n'}{end}")"

if [ -z "$nodes" ]; then
  echo "skip it.. no node candidiates.."
  exit 0
fi

for node in $nodes; do
  node_count="$(kubectl get nodes -o name | wc -l)"
  if [ "$node_count" -ge "$max_node_count" ]; then
    echo "skip it.. due to max node size reach $max_node_count"
    exit 0
  fi
  kubectl drain --delete-local-data --ignore-daemonsets --force "$node"
  kubectl wait --for=condition=ready --timeout=24h pod -l="$pod_label"
done
for node in $nodes; do
  set +e
  kubectl uncordon "$node"
  if [ "$?" -gt 0 ]; then
    echo "fail to uncordon $node. might already scale-downed?"
  fi
  set -e
done
echo done
