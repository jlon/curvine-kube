// Copyright 2025 JiangLong.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Core modules
pub mod cli;
pub mod domain;
pub mod infrastructure;
pub mod shared;

// Re-export commonly used types
pub use domain::cluster::{CurvineClusterDescriptor, KubernetesValidator};
pub use domain::config::{
    ClusterConf, KubernetesConfig, MasterConfig, ServiceConfig, ServiceType, StorageConfig,
    WorkerConfig, StorageType, WorkerDataDir,
};
pub use infrastructure::kubernetes::{CurvineKubeClient, CurvineKubeClientImpl};
pub use shared::{KubeError, Result};

// Re-export builders for internal use
#[doc(hidden)]
pub use infrastructure::kubernetes::resources::{
    ConfigMapBuilder, HeadlessServiceBuilder, MasterBuilder, ServiceBuilder, WorkerBuilder,
};
#[doc(hidden)]
pub use domain::config::KubernetesConfigBuilder;
