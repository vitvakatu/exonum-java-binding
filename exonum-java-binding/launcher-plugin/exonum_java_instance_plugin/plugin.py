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

import os
import shutil
import sys
import tempfile

from exonum_client.proofs import build_encoder_function
from exonum_client.protobuf_loader import ProtobufLoader
from exonum_client.protoc import Protoc
from exonum_launcher.instances import InstanceSpecLoader, InstanceSpecLoadError
from importlib import import_module

from exonum_launcher.configuration import Instance

_PROTOBUF_SOURCES_FIELD_NAME = "sources"
_PYTHON_MODULES_NAME = "service_modules"
_MODULE_NAME_FIELD_NAME = "module_name"
_DATA_FIELD_NAME = "data"


class JavaInstanceSpecLoader(InstanceSpecLoader):
    """Instance spec loader for Java runtime
    Compiles and loads protobuf messages provided by user to serialize service
    configuration parameters.
    """

    def load_spec(self, loader: ProtobufLoader, instance: Instance) -> bytes:
        protoc = Protoc()

        if instance.config is None:
            raise InstanceSpecLoadError(f"No config found for instance: {instance.name}")

        self.assert_config_field_exists(instance, _PROTOBUF_SOURCES_FIELD_NAME)
        self.assert_config_field_exists(instance, _MODULE_NAME_FIELD_NAME)
        self.assert_config_field_exists(instance, _DATA_FIELD_NAME)

        tmp_dir = self.prepare_tmp_dir()

        # Call protoc to compile proto sources:
        proto_output_dir = os.path.join(tmp_dir, _PYTHON_MODULES_NAME, instance.name)
        path_to_protobuf_sources = instance.config[_PROTOBUF_SOURCES_FIELD_NAME]
        protoc.compile(path_to_protobuf_sources, proto_output_dir)

        # Import module generated by protobuf
        module_name = instance.config[_MODULE_NAME_FIELD_NAME]
        service_module_path = f"{_PYTHON_MODULES_NAME}.{instance.name}.{module_name}_pb2"
        service_module = import_module(service_module_path)

        # Build encoder (serializer) for `Config` message from imported module
        config_class = service_module.Config
        config_encoder = build_encoder_function(config_class)
        result = config_encoder(instance.config[_DATA_FIELD_NAME])

        # Remove the generated temporary directory:
        self.cleanup(tmp_dir)

        return result

    @staticmethod
    def cleanup(tmp_dir):
        sys.path.remove(tmp_dir)
        shutil.rmtree(tmp_dir)

    @staticmethod
    def prepare_tmp_dir():
        # Create a directory for temporary files:
        tmp_dir = tempfile.mkdtemp(prefix="exonum_java_")
        # Create a folder for Python files output:
        python_modules_path = os.path.join(tmp_dir, _PYTHON_MODULES_NAME)
        os.makedirs(python_modules_path)
        # Create __init__ file in the service_modules directory:
        init_file_path = os.path.join(python_modules_path, "__init__.py")
        open(init_file_path, "a").close()
        # Add a directory with service_modules into the Python path:
        sys.path.append(tmp_dir)
        return tmp_dir

    @staticmethod
    def assert_config_field_exists(instance, field_name):
        if field_name not in instance.config:
            raise InstanceSpecLoadError(f"{field_name} field not found in instance {instance.name} config")
