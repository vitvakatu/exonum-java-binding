#!/usr/bin/env bash

CLASSPATH=$1
PEER_ADDRESS=$2
TEMPLATE_NAME=$3
PUB_NAME=$4
SEC_NAME=$5

CLASSPATH="${CLASSPATH}:$(cat core/ejb-core-classpath):core/classes/exonum-java-binding-common-0.4-rc-draft.jar"

JAVA_HOME="${JAVA_HOME:-$(java -XshowSettings:properties -version 2>&1 > /dev/null | grep 'java.home' | awk '{print $3}')}/"
echo "JAVA_HOME=${JAVA_HOME}"

# Find the directory containing libjvm (the relative path has changed in Java 9)
JVM_LIB_PATH="$(find ${JAVA_HOME} -type f -name libjvm.* | xargs -n1 dirname)"
echo "JVM_LIB_PATH=${JVM_LIB_PATH}"

CURRENT_DIR=$(pwd)
echo "CURRENT_DIR=${CURRENT_DIR}"

EJB_LIBPATH="${JVM_LIB_PATH}:."

echo "<====> GENERATE <====>"

LD_LIBRARY_PATH=${EJB_LIBPATH} ejb-app generate-config \
  configs/$TEMPLATE_NAME configs/$PUB_NAME configs/$SEC_NAME \
  --ejb-classpath $CLASSPATH \
  --ejb-libpath $EJB_LIBPATH \
  --ejb-log-config-path "./log_config.xml" \
  --peer-address $PEER_ADDRESS