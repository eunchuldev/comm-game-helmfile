FROM rust:latest

ARG NB_USER="jovyan"
ARG NB_UID="1000"
ARG NB_GID="100"

ENV NB_USER=$NB_USER \
    NB_UID=$NB_UID \
    NB_GID=$NB_GID \
    TOKEN=

RUN sed -i 's/^#force_color_prompt=yes/force_color_prompt=yes/' /etc/skel/.bashrc

RUN useradd -m -s /bin/bash -N -u $NB_UID $NB_USER

WORKDIR /home/$NB_USER

RUN mkdir /home/$NB_USER/work \
    && apt-get -q update \
    && apt-get install -yq --no-install-recommends \
      cmake curl xz-utils build-essential libsqlite3-dev \
      libreadline-dev libssl-dev ca-certificates openssl \
      cmake curl sudo git vim wget python3-pip python3-setuptools \
    && apt-get clean && rm -rf /var/lib/apt/lists/* \
    && ln -s /usr/bin/pip3 /usr/bin/pip \
    && echo "$NB_USER ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/$NB_USER \
    && chmod 0440 /etc/sudoers.d/$USER \
    && chown $NB_USER:$NB_GID /home/$NB_USER/work

RUN pip3 install --no-cache-dir ipykernel jupyter

RUN cargo install evcxr_jupyter cargo-edit

USER $NB_UID
RUN evcxr_jupyter --install

ENV CARGO_HOME=/home/$NB_USER/.local/lib/cargo

CMD jupyter notebook --NotebookApp.token="$TOKEN"
