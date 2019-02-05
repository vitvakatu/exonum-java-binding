FROM ubuntu:16.04

# Execute all commands using bash
SHELL ["/bin/bash", "-c"]

# Install required software
RUN apt-get update && \
  apt-get install -y --no-install-recommends software-properties-common && \
  rm -rf /var/lib/apt/lists/*
RUN add-apt-repository ppa:maarten-fonville/protobuf && \
  apt-get update && \
  apt-get install -y --no-install-recommends \
  apt-utils \
  curl \
  gcc \
  git \
  wget \
  libssl-dev \
  pkg-config \
  libsodium-dev \
  librocksdb-dev \
  libsnappy-dev \
  protobuf-compiler \
  libprotobuf-dev \
  openjdk-8-jdk \
  build-essential && \
  rm -rf /var/lib/apt/lists/*

# Install Java 8
ENV JAVA_HOME=/usr/lib/jvm/java-8-openjdk-amd64

# Install Rust
RUN curl -sSf https://sh.rustup.rs > /tmp/rustup-init.sh && \
 chmod +x /tmp/rustup-init.sh && \
 sh /tmp/rustup-init.sh -y --default-toolchain stable && \
 rm -rf /tmp/rustup-init.sh
ENV PATH="$PATH:~/.cargo/bin"

WORKDIR /app

RUN wget http://www-eu.apache.org/dist/maven/maven-3/3.6.0/binaries/apache-maven-3.6.0-bin.tar.gz
RUN tar -xf apache-maven-3.6.0-bin.tar.gz && \
  mv apache-maven-3.6.0/ apache-maven/ && \
  rm apache-maven-3.6.0-bin.tar.gz
ENV MAVEN_HOME=/app/apache-maven
ENV M2_HOME=/app/apache-maven
ENV PATH="$PATH:/app/apache-maven/bin"
ENV ROCKSDB_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV SNAPPY_LIB_DIR=/usr/lib/x86_64-linux-gnu

RUN git clone https://github.com/vitvakatu/exonum-java-binding && \
  cd /app/exonum-java-binding && \
  git checkout ecr-2789