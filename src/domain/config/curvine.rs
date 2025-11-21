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

//! Type definitions for Curvine configuration
//! This is a simplified version that keeps data structures but removes runtime dependencies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;

// ============================================================================
// Main cluster configuration
// ============================================================================

/// Main cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClusterConf {
    pub format_master: bool,
    pub format_worker: bool,
    pub testing: bool,
    pub cluster_id: String,
    pub master: MasterConf,
    pub journal: JournalConf,
    pub worker: WorkerConf,
    pub client: ClientConf,
    pub fuse: FuseConf,
    pub s3_gateway: S3GatewayConf,
    pub job: JobConf,
}

impl Default for ClusterConf {
    fn default() -> Self {
        Self {
            format_master: false,
            format_worker: false,
            testing: false,
            cluster_id: "curvine".to_string(),
            master: MasterConf::default(),
            journal: JournalConf::default(),
            worker: WorkerConf::default(),
            client: ClientConf::default(),
            fuse: FuseConf::default(),
            s3_gateway: S3GatewayConf::default(),
            job: JobConf::default(),
        }
    }
}

impl ClusterConf {
    /// Load configuration from TOML file
    pub fn from<T: AsRef<str>>(path: T) -> anyhow::Result<Self> {
        let content = read_to_string(path.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.as_ref(), e))?;

        let conf: Self =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("Failed to parse TOML: {}", e))?;

        Ok(conf)
    }
}

// ============================================================================
// Network types (替代 orpc 类型)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InetAddr {
    pub hostname: String,
    pub port: u16,
}

impl InetAddr {
    pub fn new(hostname: String, port: u16) -> Self {
        Self { hostname, port }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftPeer {
    pub id: u64,
    pub hostname: String,
    pub port: u16,
}

// ============================================================================
// Storage types
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum StorageType {
    #[serde(rename = "mem")]
    Mem,
    #[serde(rename = "ssd")]
    #[default]
    Ssd,
    #[serde(rename = "hdd")]
    Hdd,
    #[serde(rename = "disk")]
    Disk,
    #[serde(rename = "ufs")]
    Ufs,
}

impl StorageType {
    pub fn from_str_name(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mem" => Self::Mem,
            "ssd" => Self::Ssd,
            "hdd" => Self::Hdd,
            "disk" => Self::Disk,
            "ufs" => Self::Ufs,
            _ => Self::Disk,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct WorkerDataDir {
    pub storage_type: StorageType,
    pub capacity: u64,
    pub path: String,
}

impl WorkerDataDir {
    pub fn parse_data_dir(s: &str) -> anyhow::Result<Self> {
        use regex::Regex;

        let re = Regex::new(r"^\[([\w:]*)\](.+)$")?;
        let caps = match re.captures(s) {
            None => {
                return Ok(Self {
                    storage_type: StorageType::Disk,
                    capacity: 0,
                    path: s.to_string(),
                })
            }
            Some(v) => v,
        };

        let prefix = caps.get(1).map_or("", |m| m.as_str());
        let path = caps.get(2).map_or("", |m| m.as_str());
        let arr: Vec<&str> = prefix.split(':').collect();

        if prefix.is_empty() || arr.is_empty() {
            return Ok(Self {
                storage_type: StorageType::Disk,
                capacity: 0,
                path: s.to_string(),
            });
        }

        let (stg_type, capacity) = if arr.len() == 1 {
            if arr[0].chars().all(|c| c.is_alphabetic()) {
                // [HDD]/dir
                (arr[0], "0")
            } else {
                // [20GB]/dir
                ("disk", arr[0])
            }
        } else if arr.len() == 2 {
            // [HDD:20GB]/dir
            (arr[0], arr[1])
        } else {
            anyhow::bail!("Incorrect data format {}", s);
        };

        Ok(Self {
            storage_type: StorageType::from_str_name(stg_type),
            capacity: parse_size_string(capacity)?,
            path: path.to_string(),
        })
    }
}

// ============================================================================
// Master configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MasterConf {
    pub hostname: String,
    pub rpc_port: u16,
    pub web_port: u16,
    pub io_threads: usize,
    pub worker_threads: usize,
    pub io_timeout: String,
    pub io_close_idle: bool,
    pub meta_dir: String,
    pub meta_disable_wal: bool,
    pub meta_compression_type: String,
    pub meta_db_write_buffer_size: String,
    pub meta_write_buffer_size: String,
    pub min_block_size: i64,
    pub max_block_size: i64,
    pub min_replication: u16,
    pub max_replication: u16,
    pub max_path_len: usize,
    pub max_path_depth: usize,
    pub retry_cache_enable: bool,
    pub retry_cache_size: u64,
    pub retry_cache_ttl: String,
    pub block_report_limit: usize,
    pub worker_policy: String,
    pub executor_threads: usize,
    pub executor_channel_size: usize,
    pub heartbeat_interval: String,
    pub worker_check_interval: String,
    pub worker_blacklist_interval: String,
    pub worker_lost_interval: String,
    pub audit_logging_enabled: bool,
    pub block_replication_enabled: bool,
    pub block_replication_concurrency_limit: usize,
    pub ttl_checker_retry_attempts: u32,
    pub ttl_checker_interval: String,
    pub ttl_bucket_interval: String,
}

impl Default for MasterConf {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            rpc_port: 8995,
            web_port: 9000,
            io_threads: 4,
            worker_threads: 4,
            io_timeout: "10m".to_string(),
            io_close_idle: true,
            meta_dir: "/tmp/curvine/master/meta".to_string(),
            meta_disable_wal: true, // Fixed: original uses true
            meta_compression_type: "none".to_string(), // Fixed: original uses "none"
            meta_db_write_buffer_size: "0".to_string(), // Fixed: original uses "0"
            meta_write_buffer_size: "64MB".to_string(),
            min_block_size: 1024 * 1024, // Fixed: original uses 1MB
            max_block_size: 100 * 1024 * 1024 * 1024, // Fixed: original uses 100GB
            min_replication: 1,
            max_replication: 100, // Fixed: original uses 100
            max_path_len: 8000,   // Fixed: original uses 8000
            max_path_depth: 1000,
            retry_cache_enable: true,
            retry_cache_size: 100000,
            retry_cache_ttl: "10m".to_string(),
            block_report_limit: 1000,           // Fixed: original uses 1000
            worker_policy: "local".to_string(), // Fixed: original uses "local" (but "robin" is also valid)
            executor_threads: 10,               // Fixed: original uses 10
            executor_channel_size: 1000,
            heartbeat_interval: "3s".to_string(), // Fixed: original uses "3s"
            worker_check_interval: "10s".to_string(), // Fixed: original uses "10s"
            worker_blacklist_interval: "30s".to_string(), // Fixed: original uses "30s"
            worker_lost_interval: "10m".to_string(), // Fixed: original uses "10m"
            audit_logging_enabled: true,          // Fixed: original uses true
            block_replication_enabled: false,     // Fixed: original uses false
            block_replication_concurrency_limit: 1000, // Fixed: original uses 1000
            ttl_checker_retry_attempts: 3,
            ttl_checker_interval: "1h".to_string(),
            ttl_bucket_interval: "1h".to_string(),
        }
    }
}

// ============================================================================
// Journal configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct JournalConf {
    pub enable: bool,
    pub group_name: String,
    pub hostname: String,
    pub rpc_port: u16,
    pub io_threads: usize,
    pub worker_threads: usize,
    pub message_size: usize,
    /// Journal addresses - optional in k8s context (dynamically generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_addrs: Option<Vec<RaftPeer>>,
    pub journal_dir: String,
    pub writer_debug: bool,
    pub writer_channel_size: usize,
    pub writer_flush_batch_size: u64,
    pub writer_flush_batch_ms: u64,
    pub snapshot_interval: String,
    pub snapshot_entries: u64,
    pub snapshot_read_chunk_size: usize,
    pub conn_retry_max_duration_ms: u64,
    pub conn_retry_min_sleep_ms: u64,
    pub conn_retry_max_sleep_ms: u64,
    pub rpc_close_idle: bool,
    pub rpc_retry_max_duration_ms: u64,
    pub rpc_retry_min_sleep_ms: u64,
    pub rpc_retry_max_sleep_ms: u64,
    pub conn_timeout_ms: u64,
    pub io_timeout_ms: u64,
    pub conn_size: usize,
    pub raft_poll_interval_ms: u64,
    pub raft_tick_interval_ms: u64,
    pub raft_election_tick: usize,
    pub raft_heartbeat_tick: usize,
    pub raft_min_election_ticks: usize,
    pub raft_max_election_ticks: usize,
    pub raft_check_quorum: bool,
    pub raft_max_size_per_msg: u64,
    pub raft_max_inflight_msgs: usize,
    pub raft_max_committed_size_per_ready: u64,
    pub raft_retry_cache_size: u64,
    pub raft_retry_cache_ttl: String,
}

impl Default for JournalConf {
    fn default() -> Self {
        Self {
            enable: true,
            group_name: "raft-group".to_string(), // Fixed: original uses "raft-group"
            hostname: "localhost".to_string(),
            rpc_port: 8996,
            io_threads: 8,     // Fixed: original uses 8
            worker_threads: 8, // Fixed: original uses 8
            message_size: 200, // Fixed: original uses 200
            journal_addrs: None, // Optional in k8s context
            journal_dir: "/tmp/curvine/master/journal".to_string(),
            writer_debug: false,
            writer_channel_size: 0,        // Fixed: original uses 0
            writer_flush_batch_size: 1000, // Fixed: original uses 1000
            writer_flush_batch_ms: 100,
            snapshot_interval: "6h".to_string(), // Fixed: original uses "6h"
            snapshot_entries: 100000,
            snapshot_read_chunk_size: 1024 * 1024,
            conn_retry_max_duration_ms: 0,  // Fixed: original uses 0
            conn_retry_min_sleep_ms: 10000, // Fixed: original uses 10000
            conn_retry_max_sleep_ms: 10000, // Fixed: original uses 10000
            rpc_close_idle: false,
            rpc_retry_max_duration_ms: 60000, // Fixed: original uses 60000
            rpc_retry_min_sleep_ms: 20000,    // Fixed: original uses 20000
            rpc_retry_max_sleep_ms: 20000,    // Fixed: original uses 20000
            conn_timeout_ms: 30000,
            io_timeout_ms: 60000,
            conn_size: 1,
            raft_poll_interval_ms: 100,
            raft_tick_interval_ms: 1000, // Fixed: original uses 1000
            raft_election_tick: 10,      // Fixed: original uses 10
            raft_heartbeat_tick: 3,      // Fixed: original uses 3
            raft_min_election_ticks: 10, // Fixed: original uses 10
            raft_max_election_ticks: 30, // Fixed: original uses 30
            raft_check_quorum: true,
            raft_max_size_per_msg: 1024 * 1024,
            raft_max_inflight_msgs: 256,
            raft_max_committed_size_per_ready: 16 * 1024 * 1024,
            raft_retry_cache_size: 100000,
            raft_retry_cache_ttl: "10m".to_string(),
        }
    }
}

// ============================================================================
// Worker configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkerConf {
    pub hostname: String,
    pub rpc_port: u16,
    pub web_port: u16,
    pub io_threads: usize,
    pub worker_threads: usize,
    pub io_timeout: String,
    pub io_close_idle: bool,
    pub data_dir: Vec<String>,
    pub tier_alias: HashMap<String, Vec<i32>>,
    pub enable_tiered_store: bool,
    pub tiered_store_levels: Vec<String>,
    pub evict_batch_size: usize,
    pub evictor_enabled: bool,
    pub max_concurrent_read_rpcs: usize,
    pub max_concurrent_write_rpcs: usize,
    pub heartbeat_interval: String,
    pub filesystem_check_interval: String,
    pub block_remover_interval: String,
}

impl Default for WorkerConf {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            rpc_port: 8997,
            web_port: 9001,
            io_threads: 4,
            worker_threads: 4,
            io_timeout: "10m".to_string(),
            io_close_idle: false, // Fixed: original uses false
            data_dir: vec![],     // Fixed: original uses empty vec (will be set by user config)
            tier_alias: HashMap::new(),
            enable_tiered_store: false,
            tiered_store_levels: vec![],
            evict_batch_size: 1000,
            evictor_enabled: false,
            max_concurrent_read_rpcs: 100,
            max_concurrent_write_rpcs: 100,
            heartbeat_interval: "1s".to_string(),
            filesystem_check_interval: "1m".to_string(),
            block_remover_interval: "1h".to_string(),
        }
    }
}

// ============================================================================
// Client configuration
// ============================================================================

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientConf {
    pub hostname: String,
    /// Master addresses - optional in k8s context (dynamically generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_addrs: Option<Vec<InetAddr>>,
    #[serde(skip)]
    pub block_size: i64,
    #[serde(alias = "block_size")]
    pub block_size_str: String,
    pub write_type: String,
    pub read_type: String,
    pub io_retry_max_duration_ms: u64,
    pub io_retry_min_sleep_ms: u64,
    pub io_retry_max_sleep_ms: u64,
    pub worker_io_timeout_ms: u64,
    pub master_io_timeout_ms: u64,
    pub conn_size: usize,
    pub kubernetes: Option<KubernetesConf>,
}

/// Kubernetes deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KubernetesConf {
    pub namespace: String,
    pub cluster_id: Option<String>,
    pub master: KubernetesMasterConf,
    pub worker: KubernetesWorkerConf,
    pub service: KubernetesServiceConf,
    pub storage: Option<KubernetesStorageConf>,
    pub image_pull_policy: String,
    pub image_pull_secrets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KubernetesMasterConf {
    pub replicas: u32,
    pub image: String,
    pub pod_template: Option<String>,
    pub node_selector: Option<HashMap<String, String>>,
    pub graceful_shutdown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KubernetesWorkerConf {
    pub replicas: u32,
    pub image: String,
    pub pod_template: Option<String>,
    pub node_selector: Option<HashMap<String, String>>,
    pub storage_class: Option<String>,
    pub graceful_shutdown: bool,
    pub host_network: bool,
    pub init_container: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KubernetesServiceConf {
    pub service_type: String,
    pub annotations: HashMap<String, String>,
    pub session_affinity: Option<String>,
    pub external_ips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KubernetesStorageConf {
    pub storage_class: String,
    pub master_storage_class: Option<String>,
    pub worker_storage_class: Option<String>,
    pub master_size: Option<String>,
    pub worker_size: Option<String>,
}

impl Default for KubernetesConf {
    fn default() -> Self {
        Self {
            namespace: "default".to_string(),
            cluster_id: None,
            master: KubernetesMasterConf::default(),
            worker: KubernetesWorkerConf::default(),
            service: KubernetesServiceConf::default(),
            storage: None,
            image_pull_policy: "IfNotPresent".to_string(),
            image_pull_secrets: vec![],
        }
    }
}

impl Default for KubernetesMasterConf {
    fn default() -> Self {
        Self {
            replicas: 3,
            image: "docker.io/curvine:latest".to_string(),
            pod_template: None,
            node_selector: None,
            graceful_shutdown: true,
        }
    }
}

impl Default for KubernetesWorkerConf {
    fn default() -> Self {
        Self {
            replicas: 3,
            image: "docker.io/curvine:latest".to_string(),
            pod_template: None,
            node_selector: None,
            storage_class: None,
            graceful_shutdown: true,
            host_network: false,
            init_container: false,
        }
    }
}

impl Default for KubernetesServiceConf {
    fn default() -> Self {
        Self {
            service_type: "ClusterIP".to_string(),
            annotations: HashMap::new(),
            session_affinity: None,
            external_ips: vec![],
        }
    }
}

impl Default for ClientConf {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            master_addrs: None, // Optional in k8s context
            block_size: 0, // Fixed: original uses 0 (calculated from block_size_str)
            block_size_str: "128MB".to_string(), // Fixed: original uses "128MB"
            write_type: "cache_through".to_string(),
            read_type: "cache".to_string(),
            io_retry_max_duration_ms: 30000,
            io_retry_min_sleep_ms: 100,
            io_retry_max_sleep_ms: 5000,
            worker_io_timeout_ms: 60000,
            master_io_timeout_ms: 60000,
            conn_size: 1,
            kubernetes: None,
        }
    }
}

impl ClientConf {
    pub fn init(&mut self) -> anyhow::Result<()> {
        // In k8s context, master_addrs are dynamically generated, so validation is skipped
        // Only validate if master_addrs is explicitly provided and empty
        if let Some(ref addrs) = self.master_addrs {
            if addrs.is_empty() {
                anyhow::bail!("client.master_addrs cannot be empty when provided");
            }
        }
        Ok(())
    }
}

// ============================================================================
// FUSE configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FuseConf {
    pub mount_path: String,
    pub max_idle_threads: usize,
}

impl Default for FuseConf {
    fn default() -> Self {
        Self {
            mount_path: "/mnt/curvine".to_string(),
            max_idle_threads: 10,
        }
    }
}

impl FuseConf {
    pub fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ============================================================================
// Job configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct JobConf {
    pub enabled: bool,
    pub temp_folder: String,
    pub batch_size: usize,
}

impl Default for JobConf {
    fn default() -> Self {
        Self {
            enabled: false,
            temp_folder: "/tmp/curvine/job".to_string(),
            batch_size: 1000,
        }
    }
}

impl JobConf {
    pub fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ============================================================================
// S3 Gateway configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct S3GatewayConf {
    pub hostname: String,
    pub port: u16,
    pub enabled: bool,
}

impl Default for S3GatewayConf {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            port: 9002,
            enabled: false,
        }
    }
}

// ============================================================================
// Utility functions
// ============================================================================

/// Parse size string like "100GB", "1TB", "64MB"
pub fn parse_size_string(s: &str) -> anyhow::Result<u64> {
    let s = s.trim().to_uppercase();

    let (num_str, unit) = if s.ends_with("TB") {
        (&s[..s.len() - 2], 1024u64 * 1024 * 1024 * 1024)
    } else if s.ends_with("GB") {
        (&s[..s.len() - 2], 1024u64 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1024u64 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1024u64)
    } else if s.ends_with("B") {
        (&s[..s.len() - 1], 1)
    } else {
        (s.as_str(), 1)
    };

    let num: u64 = num_str
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid size string: {}", s))?;

    Ok(num * unit)
}
