FROM bitnami/postgresql:13

USER root

RUN mkdir -p /var/lib/apt/lists/partial && \
  apt-get update && \
  apt-get install --yes --no-install-recommends \
    git make build-essential && \
  git clone https://github.com/postgrespro/rum && \
  cd rum && \
  make USE_PGXS=1 install && \
  cd .. && \
  rm -rf rum && \
  apt-get remove --yes git make build-essential && \
  apt-get autoremove --yes && \
  apt clean && \
  rm -rf /var/cache/apt/* 

USER 1001

