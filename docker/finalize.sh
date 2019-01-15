#!/usr/bin/env bash

MODULE_NAME=$1
SEC_NAME=$2
FINAL_NAME=$3
EJB_PORT=$4
shift 4
PUB_CONFIGS=$@

JAVA_HOME="${JAVA_HOME:-$(java -XshowSettings:properties -version 2>&1 > /dev/null | grep 'java.home' | awk '{print $3}')}/"
echo "JAVA_HOME=${JAVA_HOME}"

# Find the directory containing libjvm (the relative path has changed in Java 9)
JVM_LIB_PATH="$(find ${JAVA_HOME} -type f -name libjvm.* | xargs -n1 dirname)"
echo "JVM_LIB_PATH=${JVM_LIB_PATH}"

CURRENT_DIR=$(pwd)
echo "CURRENT_DIR=${CURRENT_DIR}"

EJB_LIBPATH="${JVM_LIB_PATH}:."

echo "<====> FINALIZE <====>"

LD_LIBRARY_PATH=${EJB_LIBPATH} ejb-app finalize \
  configs/$SEC_NAME configs/$FINAL_NAME \
  --ejb-module-name $MODULE_NAME \
  --ejb-port $EJB_PORT \
  --public-configs $PUB_CONFIGS