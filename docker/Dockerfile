FROM ubuntu:16.04

# Execute all commands using bash
SHELL ["/bin/bash", "-c"]

# Install required software
RUN apt-get update && apt-get install -y software-properties-common
RUN add-apt-repository ppa:maarten-fonville/protobuf && \
  apt-get update && \
  apt-get install -y \
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
  build-essential && \
  rm -rf /var/lib/apt/lists/*

# Install Java 8
RUN apt-get update && \
  apt-get upgrade -y && \
  apt-get install -y  software-properties-common && \
  add-apt-repository ppa:webupd8team/java -y && \
  apt-get update && \
  echo oracle-java8-installer shared/accepted-oracle-license-v1-1 select true | /usr/bin/debconf-set-selections && \
  apt-get install -y oracle-java8-installer && \
  apt-get clean
ENV JAVA_HOME=/usr/lib/jvm/java-8-oracle

# Install Rust
RUN curl -sSf https://sh.rustup.rs > /tmp/rustup-init.sh && \
 chmod +x /tmp/rustup-init.sh && \
 sh /tmp/rustup-init.sh -y --default-toolchain nightly && \
 rm -rf /tmp/rustup-init.sh
ENV PATH="$PATH:~/.cargo/bin"

WORKDIR /app

RUN wget http://www-eu.apache.org/dist/maven/maven-3/3.5.4/binaries/apache-maven-3.5.4-bin.tar.gz
RUN tar -xf apache-maven-3.5.4-bin.tar.gz
RUN mv apache-maven-3.5.4/ apache-maven/
ENV MAVEN_HOME=/app/apache-maven
ENV M2_HOME=/app/apache-maven
ENV PATH="$PATH:/app/apache-maven/bin"
ENV ROCKSDB_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV SNAPPY_LIB_DIR=/usr/lib/x86_64-linux-gnu
RUN rm apache-maven-3.5.4-bin.tar.gz

RUN git clone https://github.com/vitvakatu/exonum-java-binding
WORKDIR /app/exonum-java-binding
RUN git checkout docker_app

RUN . ./tests_profile
RUN mvn install -DskipTests
WORKDIR /app/exonum-java-binding/exonum-java-binding-core/rust/ejb-app
RUN cargo install --path . --debug

WORKDIR /app
COPY run_ejb_app.sh .
COPY log_config.xml .
RUN mv exonum-java-binding/exonum-java-binding-core/rust/target/debug/deps/libjava_bindings.so .

EXPOSE 6001
EXPOSE 6000

ENTRYPOINT ["./run_ejb_app.sh"]