sed -i '1,/###_START_LINE_OF_DYNAMIC_CONFS_###/!d' $SPARK_HOME/conf/spark-defaults.conf

echo "\
###_START_LINE_OF_DYNAMIC_CONFS_###
spark.driver.host $(hostname -i)
spark.kubernetes.driver.pod.name ${POD_NAME:-$(hostname)}
spark.kubernetes.namespace $(cat /var/run/secrets/kubernetes.io/serviceaccount/namespace)
spark.kubernetes.container.image ${SPARK_EXECUTOR_IMAGE:-"song9446/spark:3.0.1-hadoop-3.3.0"}
" >> $SPARK_HOME/conf/spark-defaults.conf
if [[ -n $SPARK_EXECUTOR_NODE_SELECTOR_KEY ]]; then
  echo "spark.kubernetes.node.selector.$SPARK_EXECUTOR_NODE_SELECTOR_KEY $SPARK_EXECUTOR_NODE_SELECTOR_VALUE" >> $SPARK_HOME/conf/spark-defaults.conf
fi
