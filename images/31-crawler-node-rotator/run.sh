#!/bin/sh
set -e

node_count="$(kubectl get nodes -o name | wc -l)"

max_node_count="${MAX_NODE_COUNT:-1000}"

if [ "$node_count" -ge "$max_node_count" ]; then
  echo "skip it.. due to max node size reach $max_node_count"
  exit 0
fi
 
node_pool="${NODE_POOL:-crawler-pool}"

node_lifetime_min="${NODE_LIFETIME_MIN:-60}"

pod_label="${POD_LABEL:-app=dc-crawler-worker}"

min_node_creation_time="$(date --date="-${node_lifetime_min}min" -u +"%Y-%m-%dT%H:%M:%SZ")"

nodes="$(kubectl get node \
  -l cloud.google.com/gke-nodepool=$node_pool \
  --sort-by=.metadata.creationTimestamp \
  -o=jsonpath="{range .items[?(@.metadata.creationTimestamp < '${min_node_creation_time}')]}{.metadata.name}{'\n'}{end}")"

for node in $nodes; do
  kubectl drain --delete-local-data --ignore-daemonsets --force "$node"
  kubectl wait --for=condition=ready --timeout=24h pod -l="$pod_label"
done
for node in $nodes; do
  kubectl uncordon "$node"
done
echo done
