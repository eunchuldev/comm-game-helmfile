#
# Spark with very common components
#
# Included components:
# pyspark, mysql connectors
#
ARG SPARK_IMAGE=song9446/spark:3.0.1-hadoop-3.3.0
FROM ${SPARK_IMAGE}

USER root

# Setup dependencies for Google Cloud Storage access.
RUN rm $SPARK_HOME/jars/guava-29.0-jre.jar
ADD https://repo1.maven.org/maven2/com/google/guava/guava/30.0-jre/guava-30.0-jre.jar $SPARK_HOME/jars
# Add the connector jar needed to access Google Cloud Storage using the Hadoop FileSystem API.
ADD https://storage.googleapis.com/hadoop-lib/gcs/gcs-connector-latest-hadoop3.jar $SPARK_HOME/jars
ADD https://storage.googleapis.com/spark-lib/bigquery/spark-bigquery-latest_2.12.jar $SPARK_HOME/jars

# Setup dependencies for Azure Blob Storage access.
ADD https://repo1.maven.org/maven2/org/apache/hadoop/hadoop-azure/3.3.0/hadoop-azure-3.3.0.jar $SPARK_HOME/jars

# Setup for the Prometheus JMX exporter.
# Add the Prometheus JMX exporter Java agent jar for exposing metrics sent to the JmxSink to Prometheus.
ADD https://repo1.maven.org/maven2/io/prometheus/jmx/jmx_prometheus_javaagent/0.11.0/jmx_prometheus_javaagent-0.11.0.jar $SPARK_HOME/jars

# Mysql Connector
ADD https://repo1.maven.org/maven2/mysql/mysql-connector-java/8.0.11/mysql-connector-java-8.0.11.jar $SPARK_HOME/jars

RUN chmod -R o+x $SPARK_HOME/jars/

ARG APT_PACKAGES="python3-pip python3 procps"
RUN apt-get update && \
  apt-get install --no-install-recommends --yes $APT_PACKAGES && \
  rm -rf /var/cache/apt/* && \
  ln -s /usr/bin/python3 /usr/local/bin/python && \
  ln -s /usr/bin/pip3 /usr/local/bin/pip

ARG PIP_PACKAGES="setuptools pandas pyarrow"
RUN python3 -m pip install $PIP_PACKAGES

ARG PYSPARK_HOME="/usr/local/lib/python3.7/dist-packages/pyspark"
RUN cd /opt/spark/python && \
  python setup.py sdist && \
  pip install dist/* && \
  rm -r /root/.cache && \
  rm -rf dist && \
  rm -rf ${PYSPARK_HOME}/jars && \
  rm -rf ${PYSPARK_HOME}/bin && \
  rm -rf ${PYSPARK_HOME}/sbin && \
  rm -rf ${PYSPARK_HOME}/data && \
  rm -rf ${PYSPARK_HOME}/license && \
  rm -rf ${PYSPARK_HOME}/pyspark && \
  ln -s $SPARK_HOME/jars ${PYSPARK_HOME}/jars && \
  ln -s $SPARK_HOME/bin ${PYSPARK_HOME}/bin && \
  ln -s $SPARK_HOME/sbin ${PYSPARK_HOME}/sbin && \
  ln -s $SPARK_HOME/data ${PYSPARK_HOME}/data && \
  ln -s $SPARK_HOME/license ${PYSPARK_HOME}/license && \
  ln -s $SPARK_HOME/pyspark ${PYSPARK_HOME}/pyspark

ARG spark_uid=185

COPY spark-conf /opt/spark/conf
RUN chown -R ${spark_uid}:root /opt/spark/conf
RUN useradd -m -u ${spark_uid} -g root spark

USER spark

ENV PATH="/opt/spark/bin:${PATH}"

COPY test /test
COPY entrypoint.sh /opt/entrypoint.sh

# HACK IT(https://github.com/ejnkr/streamanalytics-kube-helmfile/issues/1)
USER root

ENTRYPOINT ["/opt/entrypoint.sh"]
