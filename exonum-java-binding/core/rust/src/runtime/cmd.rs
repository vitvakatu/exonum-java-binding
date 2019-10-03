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

use super::{paths::executable_directory, Config, JvmConfig, RuntimeConfig};
use exonum_cli::command::{
    finalize::Finalize, generate_config::GenerateConfig, generate_template::GenerateTemplate,
    run::Run as StandardRun, ExonumCommand, StandardResult,
};
use failure::{self, format_err};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use std::path::PathBuf;

/// All possible Exonum Java App commands.
///
/// Includes standard Exonum Core commands and modified `Run` command.
pub enum Command {
    GenerateTemplate(GenerateTemplate),
    GenerateConfig(GenerateConfig),
    Finalize(Finalize),
    Run(Run),
}

impl EjbCommand for Command {
    fn execute(self) -> Result<EjbCommandResult, failure::Error> {
        match self {
            Command::GenerateTemplate(c) => c.execute().map(|c| c.into()),
            Command::GenerateConfig(c) => c.execute().map(|c| c.into()),
            Command::Finalize(c) => c.execute().map(|c| c.into()),
            Command::Run(c) => c.execute(),
        }
    }
}

/// EJB-specific `run` command which collects standard Exonum Core parameters and
/// also additional Java runtime and JVM configuration parameters.
#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct Run {
    #[structopt(flatten)]
    #[serde(flatten)]
    standard: StandardRun,
    /// A port of the HTTP server for Java services.
    ///
    /// Must be distinct from the ports used by Exonum.
    #[structopt(long)]
    ejb_port: i32,
    /// Path to log4j configuration file.
    #[structopt(long)]
    ejb_log_config_path: Option<PathBuf>,
    /// Overrides the standard path to native libraries, enabling running the non-packaged
    /// exonum-java application.
    ///
    /// Mostly for internal usage.
    #[structopt(long)]
    ejb_override_java_library_path: Option<PathBuf>,
    /// Allows JVM being remotely debugged.
    ///
    /// Takes a socket address as a parameter in form of `HOSTNAME:PORT`.
    /// For example, `localhost:8000`
    #[structopt(long)]
    jvm_debug: Option<String>,
    /// Additional parameters for JVM that precede the rest of arguments.
    ///
    /// Must not have a leading dash. For example, `Xmx2G`.
    #[structopt(long)]
    jvm_args_prepend: Vec<String>,
    /// Additional parameters for JVM that get appended to the rest of arguments.
    ///
    /// Must not have a leading dash. For example, `Xmx2G`.
    #[structopt(long)]
    jvm_args_append: Vec<String>,
}

pub enum EjbCommandResult {
    Standard(StandardResult),
    EjbRun(Config),
}

impl From<StandardResult> for EjbCommandResult {
    fn from(result: StandardResult) -> Self {
        EjbCommandResult::Standard(result)
    }
}

pub trait EjbCommand {
    fn execute(self) -> Result<EjbCommandResult, failure::Error>;
}

impl EjbCommand for Run {
    fn execute(self) -> Result<EjbCommandResult, failure::Error> {
        if let StandardResult::Run(node_run_config) = self.standard.execute()? {
            let jvm_config = JvmConfig {
                args_prepend: self.jvm_args_prepend,
                args_append: self.jvm_args_append,
                jvm_debug_socket: self.jvm_debug,
            };

            let log_config_path = self
                .ejb_log_config_path
                .unwrap_or_else(|| get_path_to_default_log_config());

            let override_system_lib_path = self
                .ejb_override_java_library_path
                .map(|p| p.to_string_lossy().into_owned());

            let runtime_config = RuntimeConfig {
                log_config_path,
                port: self.ejb_port,
                override_system_lib_path,
            };

            let config = Config {
                standard_config: node_run_config,
                jvm_config,
                runtime_config,
            };

            Ok(EjbCommandResult::EjbRun(config))
        } else {
            Err(format_err!("Standard run command returned invalid result"))
        }
    }
}

/// Returns full path to the default log configuration file assuming the `exonum-java` app is
/// packaged/installed.
fn get_path_to_default_log_config() -> PathBuf {
    let mut path = executable_directory();
    path.push("log4j-fallback.xml");
    path
}
