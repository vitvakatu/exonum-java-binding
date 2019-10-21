# Copyright 2019 The Exonum Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from typing import Dict, Any

from exonum_launcher.runtimes.runtime import RuntimeSpecLoader

try:
    from .proto import service_runtime_pb2
except (ModuleNotFoundError, ImportError):
    raise RuntimeError("Protobuf definition is not found")


class JavaRuntimeSpecLoader(RuntimeSpecLoader):
    """Artifact spec encoder for Java runtime"""

    def encode_spec(self, data: Dict[str, Any]) -> bytes:
        spec = service_runtime_pb2.DeployArguments()

        spec.artifact_filename = data["artifact_filename"]

        return spec.SerializeToString()
