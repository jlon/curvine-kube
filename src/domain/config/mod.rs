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

//! Configuration domain

pub mod curvine;
pub mod dynamic;
pub mod kubernetes;

// Re-export Curvine configuration types
pub use self::curvine::{
    parse_size_string, ClientConf, ClusterConf, FuseConf, InetAddr, JobConf, JournalConf,
    KubernetesConf, KubernetesMasterConf, KubernetesServiceConf, KubernetesStorageConf,
    KubernetesWorkerConf, MasterConf, RaftPeer, S3GatewayConf, StorageType, WorkerConf,
    WorkerDataDir,
};

// Re-export Kubernetes configuration types
pub use self::kubernetes::{
    KubernetesConfig, KubernetesConfigBuilder, MasterConfig, ServiceConfig, ServiceType,
    StorageConfig, WorkerConfig,
};

// Re-export dynamic configuration
pub use self::dynamic::apply_to_kube_config;
