#!/usr/bin/env bash

set -eu -o pipefail

function realpath() {
  python -c 'import os, sys; print os.path.realpath(sys.argv[1])' "${1%}"
}


# Use an already set JAVA_HOME, or infer it from java.home system property.
#
# Unfortunately, a simple `which java` will not work for some users (e.g., jenv),
# hence this a bit complex thing.
JAVA_HOME="${JAVA_HOME:-$(mvn --version | grep 'Java home' | sed 's/.*: //')}"
echo "JAVA_HOME=${JAVA_HOME}"

# Find the directory containing libjvm (the relative path has changed in Java 9)
export LD_LIBRARY_PATH="$(find ${JAVA_HOME} -type f -name libjvm.* | xargs -n1 dirname)"
echo "LD_LIBRARY_PATH=${LD_LIBRARY_PATH}"

CURRENT_DIR=$(realpath ".")
echo "CURRENT_DIR=${CURRENT_DIR}"

PROJ_ROOT=$(realpath "../../..")
echo "PROJ_ROOT=${PROJ_ROOT}"

echo
echo "===[ PREPARE CLASSES ]========================================================="
echo

cd $PROJ_ROOT
mvn generate-sources
cd $CURRENT_DIR

echo
echo "===[ PREPARE PATHS ]==========================================================="
echo

CORE_TXT="exonum-java-binding-core/target/ejb-core-classpath.txt"
CRYPTOCURRENCY_TXT="exonum-java-binding-cryptocurrency-demo/target/cryptocurrency-classpath.txt"
EJB_CLASSPATH="$(cat ${PROJ_ROOT}/${CORE_TXT}):$(cat ${PROJ_ROOT}/${CRYPTOCURRENCY_TXT})"
EJB_CLASSPATH="${EJB_CLASSPATH}:${PROJ_ROOT}/exonum-java-binding-core/target/classes"
EJB_CLASSPATH="${EJB_CLASSPATH}:${PROJ_ROOT}/exonum-java-binding-cryptocurrency-demo/target/classes"
echo "EJB_CLASSPATH=${EJB_CLASSPATH}"

EJB_LIBPATH="${PROJ_ROOT}/exonum-java-binding-core/rust/target/debug"
echo "EJB_LIBPATH=${EJB_LIBPATH}"

# Clear test dir
rm -rf testnet
mkdir testnet

echo
echo "===[ GENERATE COMMON CONFIG ]=================================================="
echo

# Generate common config
cargo run -- generate-template testnet/common.toml

echo
echo "===[ GENERATE CONFIG ]========================================================="
echo

# Generate config
cargo run -- generate-config testnet/common.toml testnet/pub.toml testnet/sec.toml --ejb-classpath $EJB_CLASSPATH --ejb-libpath $EJB_LIBPATH --peer-address 127.0.0.1:5400

echo
echo "===[ FINALIZE ]================================================================"
echo

# Finalize
cargo run -- finalize testnet/sec.toml testnet/node.toml --ejb-module-name 'com.exonum.binding.cryptocurrency.ServiceModule' --ejb-port 6000 --public-configs testnet/pub.toml

echo
echo "===[ START TESTNET ]==========================================================="
echo

# Run
cargo run -- run -d testnet/db -c testnet/node.toml --public-api-address 127.0.0.1:3000
