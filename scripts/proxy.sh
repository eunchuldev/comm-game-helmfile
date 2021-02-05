ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../" >/dev/null 2>&1 && pwd )"
sudo -E $ROOT/scripts/kubefwd svc -n default -x $KUBE_CONTEXT
