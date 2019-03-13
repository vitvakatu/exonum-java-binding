/*
 * Copyright 2019 The Exonum Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use exonum::blockchain::Service;
use exonum::helpers::fabric::{Command, CommandExtension, Context, ServiceFactory};
use jni::{
    self,
    objects::{GlobalRef, JObject},
    InitArgs, InitArgsBuilder, JavaVM, Result as JniResult,
};

use std::env;
use std::fs;
use std::sync::{Arc, Once, ONCE_INIT};

use proxy::{JniExecutor, ServiceProxy};
use runtime::cmd::{Finalize, GenerateTemplate, Run};
use runtime::config::{self, Config, InternalConfig, PrivateConfig};
use utils::{
    check_error_on_exception, convert_to_string, executable_directory, join_paths, unwrap_jni,
};
use MainExecutor;

static mut JAVA_SERVICE_RUNTIME: Option<JavaServiceRuntime> = None;
static JAVA_SERVICE_RUNTIME_INIT: Once = ONCE_INIT;

const SERVICE_BOOTSTRAP_PATH: &str = "com/exonum/binding/runtime/ServiceRuntimeBootstrap";
const CREATE_RUNTIME_SIGNATURE: &str = "(I)Lcom/exonum/binding/runtime/ServiceRuntime;";
const LOAD_ARTIFACT_SIGNATURE: &str = "(Ljava/lang/String;)Ljava/lang/String;";
const CREATE_SERVICE_SIGNATURE: &str =
    "(Ljava/lang/String;Ljava/lang/String;)Lcom/exonum/binding/service/adapters/UserServiceAdapter;";

/// Controls JVM and java service.
#[allow(dead_code)]
#[derive(Clone)]
pub struct JavaServiceRuntime {
    executor: MainExecutor,
    service_runtime: GlobalRef,
}

impl JavaServiceRuntime {
    /// Creates new runtime from provided config or returns the one created earlier.
    ///
    /// There can be only one `JavaServiceRuntime` instance at a time.
    pub fn get_or_create(config: Config, internal: InternalConfig) -> Self {
        unsafe {
            // Initialize runtime if it wasn't created before.
            JAVA_SERVICE_RUNTIME_INIT.call_once(|| {
                let java_vm = Self::create_java_vm(&config.private_config, internal);
                let executor = MainExecutor::new(Arc::new(java_vm));
                let service_runtime =
                    Self::create_service_runtime(&config.private_config, executor.clone());
                let runtime = JavaServiceRuntime {
                    executor,
                    service_runtime,
                };
                JAVA_SERVICE_RUNTIME = Some(runtime);
            });
            // Return global runtime.
            JAVA_SERVICE_RUNTIME
                .clone()
                .expect("Trying to return runtime, but it's uninitialized")
        }
    }

    /// Creates a new service instance using the given artifact id.
    pub fn create_service(&self, artifact_id: &str, module: &str) -> ServiceProxy {
        unwrap_jni(self.executor.with_attached(|env| {
            let artifact_id: JObject = env.new_string(artifact_id)?.into();
            let module_name: JObject = env.new_string(module)?.into();
            let service = env
                .call_method(
                    self.service_runtime.as_obj(),
                    "createService",
                    CREATE_SERVICE_SIGNATURE,
                    &[artifact_id.into(), module_name.into()],
                )?
                .l()?;
            let service = env.new_global_ref(service).unwrap();
            Ok(ServiceProxy::from_global_ref(
                self.executor.clone(),
                service,
            ))
        }))
    }

    /// Loads an artifact from the specified location involving verification of the artifact.
    /// Returns an unique service artifact identifier that must be specified in subsequent
    /// operations with it.
    pub fn load_artifact(&self, artifact_uri: &str) -> Result<String, String> {
        unwrap_jni(self.executor.with_attached(|env| {
            let res = {
                let artifact_uri: JObject = env.new_string(artifact_uri)?.into();
                let artifact_id = env
                    .call_method(
                        self.service_runtime.as_obj(),
                        "loadArtifact",
                        LOAD_ARTIFACT_SIGNATURE,
                        &[artifact_uri.into()],
                    )?
                    .l()?;
                convert_to_string(env, artifact_id)
            };
            Ok(check_error_on_exception(env, res))
        }))
    }

    /// Initializes JVM with provided configuration.
    ///
    /// # Panics
    ///
    /// - If user specified invalid additional JVM parameters.
    fn create_java_vm(config: &PrivateConfig, internal_config: InternalConfig) -> JavaVM {
        let args = Self::build_jvm_arguments(config, internal_config)
            .expect("Unable to build arguments for JVM");
        jni::JavaVM::new(args).unwrap()
    }

    /// Builds arguments for JVM initialization.
    fn build_jvm_arguments(
        private_config: &PrivateConfig,
        internal_config: InternalConfig,
    ) -> JniResult<InitArgs> {
        let mut args_builder = jni::InitArgsBuilder::new().version(jni::JNIVersion::V8);

        let args_prepend = private_config.args_prepend.clone();
        let args_append = private_config.args_append.clone();

        // Prepend extra user arguments
        args_builder = Self::add_user_arguments(args_builder, args_prepend);

        // Add required arguments
        args_builder = Self::add_required_arguments(args_builder, private_config, internal_config);

        // Add optional arguments
        args_builder = Self::add_optional_arguments(args_builder, private_config);

        // Append extra user arguments
        args_builder = Self::add_user_arguments(args_builder, args_append);

        args_builder.build()
    }

    /// Adds extra user arguments (optional) to JVM configuration
    fn add_user_arguments<I>(mut args_builder: InitArgsBuilder, user_args: I) -> InitArgsBuilder
    where
        I: IntoIterator<Item = String>,
    {
        for param in user_args {
            let option = config::validate_and_convert(&param).unwrap();
            args_builder = args_builder.option(&option);
        }
        args_builder
    }

    /// Adds required EJB-related arguments to JVM configuration
    fn add_required_arguments(
        mut args_builder: InitArgsBuilder,
        private_config: &PrivateConfig,
        internal_config: InternalConfig,
    ) -> InitArgsBuilder {
        if internal_config.system_lib_path.is_some() {
            args_builder = args_builder.option(&format!(
                "-Djava.library.path={}",
                internal_config.system_lib_path.unwrap()
            ));
        }

        let class_path = join_paths(&[
            &internal_config.system_class_path,
            &private_config.service_class_path,
        ]);

        args_builder
            .option(&format!("-Djava.class.path={}", class_path))
            .option(&format!(
                "-Dlog4j.configurationFile={}",
                private_config.log_config_path
            ))
    }

    /// Adds optional user arguments to JVM configuration
    fn add_optional_arguments(
        mut args_builder: InitArgsBuilder,
        private_config: &PrivateConfig,
    ) -> InitArgsBuilder {
        if let Some(ref socket) = private_config.jvm_debug_socket {
            args_builder = args_builder.option(&format!(
                "-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address={}",
                socket
            ));
        }
        args_builder
    }

    /// Creates service runtime that is responsible for services management.
    fn create_service_runtime(config: &PrivateConfig, executor: MainExecutor) -> GlobalRef {
        unwrap_jni(executor.with_attached(|env| {
            let serviceRuntime = env
                .call_static_method(
                    SERVICE_BOOTSTRAP_PATH,
                    "createServiceRuntime",
                    CREATE_RUNTIME_SIGNATURE,
                    &[config.port.into()],
                )?
                .l()?;
            env.new_global_ref(serviceRuntime)
        }))
    }
}

/// Returns path to <ejb-app location>/lib directory in an absolute form.
fn absolute_library_path() -> String {
    let library_path = {
        let mut executable_directory = executable_directory();
        executable_directory.push("lib/native");
        executable_directory
    };
    library_path.to_string_lossy().into_owned()
}

fn system_classpath() -> String {
    let mut jars = Vec::new();
    let jars_directory = {
        let mut executable_directory = executable_directory();
        executable_directory.push("lib/java");
        executable_directory
    };
    for entry in fs::read_dir(jars_directory).expect("Could not read java classes directory") {
        let file = entry.unwrap();
        if file.file_type().unwrap().is_file() {
            jars.push(file.path());
        } else {
            continue;
        }
    }

    let jars = jars.iter().map(|p| p.to_str().unwrap());
    env::join_paths(jars).unwrap().into_string().unwrap()
}

/// Panics if `_JAVA_OPTIONS` environmental variable is set.
pub fn panic_if_java_options() {
    if env::var("_JAVA_OPTIONS").is_ok() {
        panic!(
            "_JAVA_OPTIONS environment variable is set. \
             Due to the fact that it will overwrite any JVM settings, \
             including ones set by EJB internally, this variable is \
             forbidden for EJB applications.\n\
             It is recommended to use `--jvm-args-append` and `--jvm-args-prepend` command-line \
             parameters for setting custom JVM parameters."
        );
    }
}

/// Factory for particular Java service.
/// Initializes EJB runtime and creates `ServiceProxy`.
pub struct JavaServiceFactory;

impl ServiceFactory for JavaServiceFactory {
    fn service_name(&self) -> &str {
        "JAVA_SERVICE_FACTORY"
    }

    fn command(&mut self, command: &str) -> Option<Box<CommandExtension>> {
        use exonum::helpers::fabric;
        // Execute EJB configuration steps along with standard Exonum Core steps.
        match command {
            v if v == fabric::GenerateCommonConfig.name() => Some(Box::new(GenerateTemplate)),
            v if v == fabric::Finalize.name() => Some(Box::new(Finalize)),
            v if v == fabric::Run.name() => Some(Box::new(Run)),
            _ => None,
        }
    }

    fn make_service(&mut self, context: &Context) -> Box<Service> {
        use exonum::helpers::fabric::keys;
        let config: Config = context
            .get(keys::NODE_CONFIG)
            .expect("Unable to read node configuration.")
            .services_configs
            .get(super::cmd::EJB_CONFIG_SECTION_NAME)
            .expect("Unable to read EJB configuration.")
            .clone()
            .try_into()
            .expect("Invalid EJB configuration format.");

        let internal_config = InternalConfig {
            system_class_path: system_classpath(),
            system_lib_path: Some(absolute_library_path()),
        };

        let runtime = JavaServiceRuntime::get_or_create(config.clone(), internal_config);

        let service_proxy = runtime.create_service("", &config.public_config.module_name);
        Box::new(service_proxy)
    }
}
