sed -i '1,/###_START_LINE_OF_DYNAMIC_CONFS_###/!d' $SPARK_HOME/conf/spark-defaults.conf

echo "\
###_START_LINE_OF_DYNAMIC_CONFS_###
spark.driver.host $(hostname -i)
spark.kubernetes.driver.pod.name ${POD_NAME:-$(hostname)}
spark.kubernetes.namespace $(cat /var/run/secrets/kubernetes.io/serviceaccount/namespace)
spark.kubernetes.container.image ${SPARK_EXECUTOR_IMAGE:-"song9446/spark:3.0.1-hadoop-3.3.0"}
spark.driver.extraJavaOptions -javaagent:$SPARK_HOME/jars/jmx_prometheus_javaagent-0.11.0.jar=9091:$SPARK_HOME/conf/prometheus.yaml
spark.executor.extraJavaOptions -javaagent:$SPARK_HOME/jars/jmx_prometheus_javaagent-0.11.0.jar=9091:$SPARK_HOME/conf/prometheus.yaml
#spark.kubernetes.executor.annotation.prometheus.io/scrape true
#spark.kubernetes.executor.annotation.prometheus.io/port 9091
#spark.kubernetes.executor.annotation.prometheus.io/path /
spark.kubernetes.driver.annotation.prometheus.io/scrape true
spark.kubernetes.driver.annotation.prometheus.io/port 4040
spark.kubernetes.driver.annotation.prometheus.io/path /metrics/executors/prometheus/
" >> $SPARK_HOME/conf/spark-defaults.conf
if [[ -n $SPARK_EXECUTOR_NODE_SELECTOR_KEY ]]; then
  echo "spark.kubernetes.node.selector.$SPARK_EXECUTOR_NODE_SELECTOR_KEY $SPARK_EXECUTOR_NODE_SELECTOR_VALUE" >> $SPARK_HOME/conf/spark-defaults.conf
fi
if [[ -n "$SPARK_EXTRA_CONFIGS" ]]; then
  for config in $SPARK_EXTRA_CONFIGS; do
    echo "$config" | sed '1,/=/{s/=/ /;}' >> "$SPARK_HOME/conf/spark-defaults.conf"
  done
fi
