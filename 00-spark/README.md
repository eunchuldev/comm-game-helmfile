# SPARK FOR K8S

## ENV VARS
 * POD_NAME                           - Optional. If not set, the hostname is supposed to be the pod name. You have to manally set this env var if pod name is too long.
 * SPARK_EXECUTOR_IMAGE               - Optional. 
 * SPARK_EXECUTOR_NODE_SELECTOR_KEY   - Optional.
 * SPARK_EXECUTOR_NODE_SELECTOR_VALUE - Optional. If SPARK_EXECUTOR_NODE_SELECTOR_KEY is set, it also must be set.
