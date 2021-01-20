#!/bin/sh
set -e

max_node_count="${MAX_NODE_COUNT:-1000}"

node_lifetime_min="${NODE_LIFETIME_MIN:-2}"

pod_label="${POD_LABEL:-app.kubernetes.io/name=dc-crawler-worker}"

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

node_max_cpu="$(kubectl get node -l cloud.google.com/gke-nodepool=crawler-pool --sort-by=.metadata.creationTimestamp -o=jsonpath='{.items[0].status.allocatable.cpu}')"
node_max_memory="$(kubectl get node -l cloud.google.com/gke-nodepool=crawler-pool --sort-by=.metadata.creationTimestamp -o=jsonpath='{.items[0].status.allocatable.memory}')"
cat >dummy-pod.yaml <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: dummy-1
  labels:
    role: dummy
spec:
  nodeSelector:
    ${node_selector/=/: }
  containers:
    - name: main
      image: busybox:stable
      resources:
        requests:
          memory: "$(( ${node_max_memory::(-2)} * 8 / 10 ))Ki"
          cpu: "$(( ${node_max_cpu::(-1)} * 7 / 10 ))m"
      command: [ sh ]
      args:
        - -c
        - while true; do sleep 2; done
EOF

for node in $nodes; do
  node_count="$(kubectl get nodes -o name | wc -l)"
  if [ "$node_count" -ge "$max_node_count" ]; then
    echo "skip it.. due to max node size reach $max_node_count"
    exit 0
  fi
  echo "1. create dummy pod to trigger autoscale up"
  kubectl create -f dummy-pod.yaml
  kubectl wait --for=condition=ready --timeout=24h pod -l="role=dummy"
  echo ""
  echo "2. scale up done. remove dummy pod"
  kubectl delete pod -l="role=dummy" --wait=true
  echo ""
  echo "3. drain expired node. wait until all crawlers ready"
  kubectl drain --delete-local-data --ignore-daemonsets --force "$node"
  kubectl wait --for=condition=ready --timeout=24h pod -l="$pod_label"
  echo ""
  echo "4. wait until node deleted(might takes 10~30mins)"
  until kubectl get node $node 2>&1 >/dev/null; do sleep 10; do
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
