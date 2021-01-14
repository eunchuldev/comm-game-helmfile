FROM spark

FROM base-notebook

USER root
RUN apt-get update && apt-get install --yes --no-install-recommends \
    dnsutils \
    git \
    iputils-ping \
    openjdk-11-jdk \
    build-essential \
 && rm -rf /var/lib/apt/lists/*

COPY --from=spark /opt/spark /opt/spark
COPY --from=spark /usr/local/lib/python3.7/dist-packages/pyspark /opt/conda/lib/python3.7/pyspark

RUN chown -R $NB_USER:1000 /opt/spark/conf/spark-defaults.conf && \
  echo 'echo "spark.ui.proxyBase ${JUPYTERHUB_SERVICE_PREFIX:-jovyan/}proxy/4040" >> $SPARK_HOME/conf/spark-defaults.conf' >> /opt/spark/conf/spark-env.sh

ENV SPARK_HOME=/opt/spark

USER $NB_USER

COPY requirements.txt /tmp/requirements.txt
RUN python -m pip install --no-cache-dir -r /tmp/requirements.txt && \
  jupyter serverextension enable --py nbgitpuller --sys-prefix && \
  jupyter serverextension enable --sys-prefix jupyter_server_proxy


ENV NBGITPULLER_APP=lab

USER $NB_USER
