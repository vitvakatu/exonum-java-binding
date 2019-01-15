#!/usr/bin/env bash

NODE_COUNT=$1

TEMPLATE_NAME=$2

JAVA_HOME="${JAVA_HOME:-$(java -XshowSettings:properties -version 2>&1 > /dev/null | grep 'java.home' | awk '{print $3}')}/"
echo "JAVA_HOME=${JAVA_HOME}"

# Find the directory containing libjvm (the relative path has changed in Java 9)
JVM_LIB_PATH="$(find ${JAVA_HOME} -type f -name libjvm.* | xargs -n1 dirname)"
echo "JVM_LIB_PATH=${JVM_LIB_PATH}"

CURRENT_DIR=$(pwd)
echo "CURRENT_DIR=${CURRENT_DIR}"

echo "<====> TEMPLATE (Node count = $1) <====>"

EJB_LIBPATH="${JVM_LIB_PATH}:."

LD_LIBRARY_PATH=${EJB_LIBPATH} ejb-app generate-template \
  configs/$TEMPLATE_NAME \
  --validators-count $NODE_COUNT