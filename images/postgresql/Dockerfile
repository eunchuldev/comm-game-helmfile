FROM bitnami/postgresql:13

USER root

RUN mkdir -p /var/lib/apt/lists/partial && \
  apt-get update && \
  apt-get install --yes --no-install-recommends \
    git make build-essential libicu-dev && \
  git clone https://github.com/postgrespro/rum && \
  cd rum && \
  make USE_PGXS=1 install && \
  cd .. && \
  rm -rf rum && \
  git clone https://github.com/akorotkov/vgram && \
  cd vgram && \
  make USE_PGXS=1 install && \
  cd .. && \
  rm -rf vgram && \
  apt-get remove --yes git make build-essential && \
  apt-get autoremove --yes && \
  apt clean && \
  rm -rf /var/cache/apt/* 

USER 1001

