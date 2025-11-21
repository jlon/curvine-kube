//! Kubernetes deployment commands

use crate::domain::config::ClusterConf;
use crate::{
    CurvineClusterDescriptor, KubernetesConfig, MasterConfig, ServiceConfig, ServiceType,
    StorageConfig, WorkerConfig,
};
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser, Debug, Clone)]
pub struct DeployCommand {
    /// Cluster ID (must be a valid Kubernetes name)
    /// If not provided, will use client.kubernetes.cluster_id from config file
    #[arg(long, short = 'c')]
    pub cluster_id: Option<String>,

    /// Kubernetes namespace
    #[arg(long, short = 'n', default_value = "default")]
    pub namespace: String,

    /// Path to kubeconfig file
    /// If not specified, uses default kubeconfig resolution (KUBECONFIG env or ~/.kube/config)
    #[arg(long)]
    pub kubeconfig: Option<String>,

    /// Kubernetes context to use
    /// If not specified, uses current context from kubeconfig
    #[arg(long)]
    pub context: Option<String>,

    /// Image for both master and worker
    #[arg(long, default_value = "docker.io/curvine:latest")]
    pub image: String,

    /// Master replicas (must be odd for Raft)
    #[arg(long, default_value = "3")]
    pub master_replicas: u32,

    /// Worker replicas
    #[arg(long, default_value = "3")]
    pub worker_replicas: u32,

    /// Master Pod template file path (YAML)
    #[arg(long)]
    pub master_pod_template: Option<String>,

    /// Worker Pod template file path (YAML)
    #[arg(long)]
    pub worker_pod_template: Option<String>,

    /// Storage class for PVCs (both master and worker, unless master-storage-class or worker-storage-class is specified)
    #[arg(long)]
    pub storage_class: Option<String>,

    /// Master storage class (overrides storage-class for master)
    #[arg(long)]
    pub master_storage_class: Option<String>,

    /// Worker storage class (overrides storage-class for worker)
    #[arg(long)]
    pub worker_storage_class: Option<String>,

    /// Master storage size (e.g., "100Gi")
    #[arg(long)]
    pub master_storage_size: Option<String>,

    /// Worker storage size (e.g., "100Gi")
    #[arg(long)]
    pub worker_storage_size: Option<String>,

    /// Service type (ClusterIP, NodePort, LoadBalancer)
    #[arg(long, default_value = "ClusterIP")]
    pub service_type: String,

    /// Image pull policy
    #[arg(long, default_value = "IfNotPresent")]
    pub image_pull_policy: String,

    /// Path to Curvine cluster configuration file (curvine-cluster.toml)
    /// If not provided, only Kubernetes deployment will be created (no Curvine config)
    #[arg(long, value_name = "PATH")]
    pub config_file: Option<String>,

    /// Dynamic configuration properties to override any settings (-D key=value)
    ///
    /// Basic: kubernetes.namespace, kubernetes.cluster-id, kubernetes.config.file, kubernetes.context
    /// Images: kubernetes.container.image (sets both master and worker), kubernetes.image.pull-policy
    /// Replicas: kubernetes.master.replicas, kubernetes.worker.replicas
    /// Resources: kubernetes.master.cpu, kubernetes.master.memory, kubernetes.worker.cpu, kubernetes.worker.memory
    /// Node Scheduling: kubernetes.master.node-selector, kubernetes.worker.node-selector (format: key1=val1,key2=val2)
    /// Labels: kubernetes.master.labels, kubernetes.worker.labels (format: key1=val1,key2=val2)
    /// Annotations: kubernetes.master.annotations, kubernetes.worker.annotations, kubernetes.service.annotations
    /// Security: kubernetes.master.service-account, kubernetes.worker.service-account
    /// Environment: kubernetes.master.env.VAR_NAME, kubernetes.worker.env.VAR_NAME (dynamic env vars)
    /// DNS & Priority: kubernetes.pod.dns-policy, kubernetes.pod.priority-class
    /// Storage: kubernetes.storage.class, kubernetes.storage.size
    /// Service: kubernetes.service.type, kubernetes.service.external-ips (comma-separated)
    /// Templates: kubernetes.master.pod-template, kubernetes.worker.pod-template (for complex config like tolerations)
    ///
    /// Example: -Dkubernetes.master.cpu=2.0 -Dkubernetes.master.labels=app=curvine,tier=master
    #[arg(short = 'D', value_name = "KEY=VALUE")]
    pub properties: Vec<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct UpdateCommand {
    #[arg(long, short = 'c')]
    pub cluster_id: Option<String>,

    #[arg(long, short = 'n', default_value = "default")]
    pub namespace: String,

    #[arg(long)]
    pub kubeconfig: Option<String>,

    #[arg(long)]
    pub context: Option<String>,

    /// Path to Curvine cluster configuration file (curvine-cluster.toml)
    /// If flag is used without value, reads from CURVINE_CONF_FILE environment variable
    /// If path is specified, uses that path to update ConfigMap
    /// If flag is not used, ConfigMap is not updated
    #[arg(long, num_args = 0..=1, default_missing_value = "${CURVINE_CONF_FILE}", value_name = "PATH")]
    pub config_file: Option<String>,

    /// Image for both master and worker
    #[arg(long)]
    pub image: Option<String>,

    #[arg(long)]
    pub master_replicas: Option<u32>,

    #[arg(long)]
    pub worker_replicas: Option<u32>,

    #[arg(long)]
    pub master_pod_template: Option<String>,

    #[arg(long)]
    pub worker_pod_template: Option<String>,

    #[arg(long)]
    pub service_type: Option<String>,

    #[arg(long)]
    pub image_pull_policy: Option<String>,

    #[arg(short = 'D', value_name = "KEY=VALUE")]
    pub properties: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ListCommand {
    #[arg(long, short = 'n')]
    pub namespace: Option<String>,
}

#[derive(Parser, Debug)]
pub struct StatusCommand {
    /// Cluster ID
    #[arg(long, short = 'c')]
    pub cluster_id: Option<String>,

    /// Kubernetes namespace
    #[arg(long, short = 'n', default_value = "default")]
    pub namespace: String,

    /// Kubeconfig file path
    #[arg(long)]
    pub kubeconfig: Option<String>,

    /// Kubernetes context
    #[arg(long)]
    pub context: Option<String>,
}

#[derive(Parser, Debug)]
pub struct DeleteCommand {
    /// Cluster ID
    #[arg(long, short = 'c')]
    pub cluster_id: Option<String>,

    /// Kubernetes namespace
    #[arg(long, short = 'n', default_value = "default")]
    pub namespace: String,

    /// Delete PVCs (persistent volumes)
    #[arg(long)]
    pub delete_pvcs: bool,

    /// Kubeconfig file path
    #[arg(long)]
    pub kubeconfig: Option<String>,

    /// Kubernetes context
    #[arg(long)]
    pub context: Option<String>,
}

impl DeployCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        // Load cluster configuration - optional, use defaults if not provided
        let cluster_conf = if let Some(ref config_path) = self.config_file {
            ClusterConf::from(config_path)?
        } else if let Ok(env_path) = std::env::var("CURVINE_CONF_FILE") {
            // Try to use CURVINE_CONF_FILE environment variable if set
            ClusterConf::from(&env_path)?
        } else {
            // Use default configuration if no config file is provided
            println!("ℹ️  No configuration file specified, using default settings");
            ClusterConf::default()
        };
        // Parse dynamic configurations
        let dynamic_configs = if !self.properties.is_empty() {
            Some(
                parse_dynamic_configs(&self.properties)
                    .map_err(|e| anyhow::anyhow!("Failed to parse dynamic configs: {}", e))?,
            )
        } else {
            None
        };

        // Note: Dynamic configs are now applied directly to KubernetesConfig
        // via crate::domain::config::dynamic::apply_to_kube_config() later in this function.
        let cmd = self;

        // Get Kubernetes configuration from ClientConf.kubernetes
        // Priority: command line > config file > defaults
        let kube_conf = cluster_conf.client.kubernetes.as_ref();

        // Resolve cluster_id: command line > config file > error
        let cluster_id = cmd
            .cluster_id
            .clone()
            .or_else(|| kube_conf.and_then(|k| k.cluster_id.clone()))
            .ok_or_else(|| anyhow::anyhow!("cluster_id is required (use --cluster-id)"))?;

        // Resolve namespace: command line > config file > default
        let namespace = if !cmd.namespace.is_empty() && cmd.namespace != "default" {
            cmd.namespace.clone()
        } else {
            kube_conf
                .map(|k| k.namespace.clone())
                .unwrap_or_else(|| cmd.namespace.clone())
        };

        // Resolve service type: command line > config file > default
        let service_type_str = if cmd.service_type != "ClusterIP" {
            cmd.service_type.clone()
        } else {
            kube_conf
                .map(|k| k.service.service_type.clone())
                .unwrap_or_else(|| cmd.service_type.clone())
        };
        let service_type = service_type_str
            .parse::<ServiceType>()
            .map_err(|e| anyhow::anyhow!("Invalid service type: {}", e))?;

        // Resolve master config: command line > config file > defaults
        let master_replicas = if cmd.master_replicas != 3 {
            cmd.master_replicas
        } else {
            kube_conf
                .map(|k| k.master.replicas)
                .unwrap_or(cmd.master_replicas)
        };

        // Priority: --image > config file > default
        let master_image = if cmd.image != "docker.io/curvine:latest" {
            cmd.image.clone()
        } else {
            kube_conf
                .map(|k| k.master.image.clone())
                .unwrap_or_else(|| cmd.image.clone())
        };

        let master_pod_template = cmd
            .master_pod_template
            .clone()
            .or_else(|| kube_conf.and_then(|k| k.master.pod_template.clone()));

        let master_node_selector = kube_conf.and_then(|k| k.master.node_selector.clone());

        // Resolve worker config: command line > config file > defaults
        let worker_replicas = if cmd.worker_replicas != 3 {
            cmd.worker_replicas
        } else {
            kube_conf
                .map(|k| k.worker.replicas)
                .unwrap_or(cmd.worker_replicas)
        };

        // Priority: --image > config file > default
        let worker_image = if cmd.image != "docker.io/curvine:latest" {
            cmd.image.clone()
        } else {
            kube_conf
                .map(|k| k.worker.image.clone())
                .unwrap_or_else(|| cmd.image.clone())
        };

        let worker_pod_template = cmd
            .worker_pod_template
            .clone()
            .or_else(|| kube_conf.and_then(|k| k.worker.pod_template.clone()));

        let worker_node_selector = kube_conf.and_then(|k| k.worker.node_selector.clone());

        // Resolve storage config: command line > config file
        let storage_class = cmd
            .storage_class
            .clone()
            .or_else(|| kube_conf.and_then(|k| k.worker.storage_class.clone()))
            .or_else(|| {
                kube_conf.and_then(|k| k.storage.as_ref().map(|s| s.storage_class.clone()))
            });

        let master_storage_size = cmd.master_storage_size.clone().or_else(|| {
            kube_conf.and_then(|k| k.storage.as_ref().and_then(|s| s.master_size.clone()))
        });

        let worker_storage_size = cmd.worker_storage_size.clone().or_else(|| {
            kube_conf.and_then(|k| k.storage.as_ref().and_then(|s| s.worker_size.clone()))
        });

        // Resolve image pull policy: command line > config file > default
        let image_pull_policy = if cmd.image_pull_policy != "IfNotPresent" {
            cmd.image_pull_policy.clone()
        } else {
            kube_conf
                .map(|k| k.image_pull_policy.clone())
                .unwrap_or_else(|| cmd.image_pull_policy.clone())
        };

        // Build Kubernetes configuration
        let mut kube_config = KubernetesConfig {
            cluster_id,
            namespace: namespace.clone(),
            master: MasterConfig {
                replicas: master_replicas,
                image: master_image,
                resources: None,
                node_selector: master_node_selector,
                affinity: None,
                pod_template: master_pod_template,
                graceful_shutdown: kube_conf
                    .map(|k| k.master.graceful_shutdown)
                    .unwrap_or(true),
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tolerations: Vec::new(),
                service_account: None,
                env_vars: HashMap::new(),
                dns_policy: None,
                priority_class: None,
            },
            worker: WorkerConfig {
                replicas: worker_replicas,
                image: worker_image,
                resources: None,
                node_selector: worker_node_selector,
                anti_affinity: false,
                pod_template: worker_pod_template,
                storage_class: storage_class.clone(),
                graceful_shutdown: kube_conf
                    .map(|k| k.worker.graceful_shutdown)
                    .unwrap_or(true),
                host_network: kube_conf.map(|k| k.worker.host_network).unwrap_or(false),
                init_container: kube_conf.map(|k| k.worker.init_container).unwrap_or(false),
                host_path_storage: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tolerations: Vec::new(),
                service_account: None,
                env_vars: HashMap::new(),
                dns_policy: None,
                priority_class: None,
            },
            service: ServiceConfig {
                service_type,
                annotations: kube_conf
                    .map(|k| k.service.annotations.clone())
                    .unwrap_or_default(),
                session_affinity: kube_conf.and_then(|k| k.service.session_affinity.clone()),
                external_ips: kube_conf
                    .map(|k| k.service.external_ips.clone())
                    .unwrap_or_default(),
                load_balancer_source_ranges: Vec::new(),
            },
            storage: storage_class
                .as_ref()
                .or(cmd.master_storage_class.as_ref())
                .or(cmd.worker_storage_class.as_ref())
                .or(master_storage_size.as_ref())
                .or(worker_storage_size.as_ref())
                .map(|_| {
                    let fallback_sc = cmd
                        .master_storage_class
                        .as_ref()
                        .or(cmd.worker_storage_class.as_ref())
                        .cloned()
                        .unwrap_or_default();
                    StorageConfig {
                        storage_class: storage_class.clone().unwrap_or(fallback_sc),
                        master_storage_class: cmd.master_storage_class.clone(),
                        worker_storage_class: cmd.worker_storage_class.clone(),
                        master_size: master_storage_size.clone(),
                        worker_size: worker_storage_size.clone(),
                    }
                }),
            image_pull_policy,
            image_pull_secrets: kube_conf
                .map(|k| k.image_pull_secrets.clone())
                .unwrap_or_default(),
            cluster_domain: "cluster.local".to_string(),
        };

        // Apply advanced dynamic configurations directly to kube_config
        if let Some(ref configs) = dynamic_configs {
            crate::domain::config::dynamic::apply_to_kube_config(configs, &mut kube_config);
        }

        // Create cluster descriptor with kubeconfig options
        let descriptor = CurvineClusterDescriptor::new_with_config(
            namespace,
            cmd.kubeconfig.clone(),
            cmd.context.clone(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create cluster descriptor: {}", e))?;

        // Deploy cluster
        descriptor
            .deploy_cluster(&cluster_conf, &kube_config)
            .await
            .map_err(|e| {
                // Clean up error formatting - remove duplicate prefixes
                let error_msg = e.to_string();
                if error_msg.starts_with("Failed to deploy cluster: ") {
                    anyhow::anyhow!("{}", error_msg)
                } else {
                    anyhow::anyhow!("Deployment failed: {}", error_msg)
                }
            })?;

        println!("Cluster {} deployed successfully!", kube_config.cluster_id);
        Ok(())
    }
}

impl UpdateCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        // Load cluster configuration - required
        let cluster_conf = if let Some(ref config_file_path) = self.config_file {
            // Handle ${CURVINE_CONF_FILE} placeholder
            let actual_path = if config_file_path == "${CURVINE_CONF_FILE}" {
                std::env::var("CURVINE_CONF_FILE").map_err(|_| {
                    anyhow::anyhow!("❌ CURVINE_CONF_FILE environment variable not set\n\n  Usage:\n  export CURVINE_CONF_FILE=/path/to/your/curvine-cluster.toml\n  curvine-kube update -c test --config-file")
                })?
            } else {
                config_file_path.clone()
            };

            // Validate file exists
            if !std::path::Path::new(&actual_path).exists() {
                anyhow::bail!("❌ Configuration file not found: {}", actual_path);
            }

            // Load the new configuration
            let conf = ClusterConf::from(&actual_path).map_err(|e| {
                anyhow::anyhow!("Failed to load configuration from {}: {}", actual_path, e)
            })?;

            println!("✓ Loaded new configuration from: {}", actual_path);
            println!("  Note: Dynamic parameters (master addresses, journal addresses) will be regenerated from cluster state");

            conf
        } else {
            // Try to use CURVINE_CONF_FILE environment variable
            let env_path = std::env::var("CURVINE_CONF_FILE")
                .map_err(|_| anyhow::anyhow!(
                    "Configuration file is required. Please specify --config-file or set CURVINE_CONF_FILE environment variable"
                ))?;
            ClusterConf::from(&env_path)?
        };

        // Check if master_replicas is being updated (not supported)
        if self.master_replicas.is_some() {
            anyhow::bail!(
                "❌ Master replicas cannot be updated\n\n  Reason: Master nodes form a Raft cluster and changing replica count requires manual cluster reconfiguration.\n\n💡 Supported updates:\n- Worker replicas: --worker-replicas <count>\n- Master image: --master-image <image>\n- Worker image: --worker-image <image>\n- Image pull policy: --image-pull-policy <policy>\n- Service type: --service-type <type>\n- Pod templates and resources via -D\n\n📝 Example: curvine-kube update -c test --worker-replicas 5"
            );
        }

        // Parse dynamic configurations
        let dynamic_configs = if !self.properties.is_empty() {
            Some(
                parse_dynamic_configs(&self.properties)
                    .map_err(|e| anyhow::anyhow!("Failed to parse dynamic configs: {}", e))?,
            )
        } else {
            None
        };

        // Note: Dynamic configs are now applied directly to KubernetesConfig
        // via crate::domain::config::dynamic::apply_to_kube_config() later in this function.
        let cmd = self;

        // Get Kubernetes configuration from ClientConf.kubernetes or use command line arguments
        let kube_conf = cluster_conf.client.kubernetes.as_ref();

        // Resolve cluster_id: command line > config file > error
        let cluster_id = cmd
            .cluster_id
            .clone()
            .or_else(|| kube_conf.and_then(|k| k.cluster_id.clone()))
            .ok_or_else(|| anyhow::anyhow!("cluster_id is required (use --cluster-id)"))?;

        // Resolve namespace: command line > config file > default
        let namespace = if !cmd.namespace.is_empty() && cmd.namespace != "default" {
            cmd.namespace.clone()
        } else {
            kube_conf
                .map(|k| k.namespace.clone())
                .unwrap_or_else(|| cmd.namespace.clone())
        };

        // Resolve service type: only use if explicitly provided
        let service_type_str = cmd
            .service_type
            .clone()
            .or_else(|| kube_conf.map(|k| k.service.service_type.clone()));
        let service_type = if let Some(st) = service_type_str {
            st.parse::<ServiceType>()
                .map_err(|e| anyhow::anyhow!("Invalid service type: {}", e))?
        } else {
            ServiceType::ClusterIP
        };

        let master_replicas = kube_conf.map(|k| k.master.replicas).unwrap_or(1) as i32;

        // Priority: --image > config file
        let master_image = cmd
            .image
            .clone()
            .or_else(|| kube_conf.map(|k| k.master.image.clone()));

        let master_pod_template = cmd
            .master_pod_template
            .clone()
            .or_else(|| kube_conf.and_then(|k| k.master.pod_template.clone()));

        let master_node_selector = kube_conf.and_then(|k| k.master.node_selector.clone());

        // Resolve worker config: only update if explicitly provided
        let worker_replicas = cmd
            .worker_replicas
            .or_else(|| kube_conf.map(|k| k.worker.replicas));

        // Priority: --image > config file
        let worker_image = cmd
            .image
            .clone()
            .or_else(|| kube_conf.map(|k| k.worker.image.clone()));

        let worker_pod_template = cmd
            .worker_pod_template
            .clone()
            .or_else(|| kube_conf.and_then(|k| k.worker.pod_template.clone()));

        let worker_node_selector = kube_conf.and_then(|k| k.worker.node_selector.clone());

        // Note: storage_class and storage_size are not updatable
        // They only affect new PVCs, and existing PVCs cannot be modified in K8s
        // Use the existing storage config from cluster
        let storage_config = kube_conf.and_then(|k| {
            k.storage.as_ref().map(|s| StorageConfig {
                storage_class: s.storage_class.clone(),
                master_storage_class: s.master_storage_class.clone(),
                worker_storage_class: s.worker_storage_class.clone(),
                master_size: s.master_size.clone(),
                worker_size: s.worker_size.clone(),
            })
        });

        // Resolve image pull policy: only update if explicitly provided
        let image_pull_policy = cmd
            .image_pull_policy
            .clone()
            .or_else(|| kube_conf.map(|k| k.image_pull_policy.clone()));

        // Build Kubernetes configuration with optional fields
        let mut kube_config = KubernetesConfig {
            cluster_id,
            namespace: namespace.clone(),
            master: MasterConfig {
                replicas: master_replicas as u32,
                image: master_image.unwrap_or_else(|| "docker.io/curvine:latest".to_string()),
                resources: None,
                node_selector: master_node_selector,
                affinity: None,
                pod_template: master_pod_template,
                graceful_shutdown: kube_conf
                    .map(|k| k.master.graceful_shutdown)
                    .unwrap_or(true),
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tolerations: Vec::new(),
                service_account: None,
                env_vars: HashMap::new(),
                dns_policy: None,
                priority_class: None,
            },
            worker: WorkerConfig {
                replicas: worker_replicas.unwrap_or(3),
                image: worker_image.unwrap_or_else(|| "docker.io/curvine:latest".to_string()),
                resources: None,
                node_selector: worker_node_selector,
                anti_affinity: false,
                pod_template: worker_pod_template,
                storage_class: storage_config.as_ref().map(|s| s.storage_class.clone()),
                graceful_shutdown: kube_conf
                    .map(|k| k.worker.graceful_shutdown)
                    .unwrap_or(true),
                host_network: kube_conf.map(|k| k.worker.host_network).unwrap_or(false),
                init_container: kube_conf.map(|k| k.worker.init_container).unwrap_or(false),
                host_path_storage: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tolerations: Vec::new(),
                service_account: None,
                env_vars: HashMap::new(),
                dns_policy: None,
                priority_class: None,
            },
            service: ServiceConfig {
                service_type,
                annotations: kube_conf
                    .map(|k| k.service.annotations.clone())
                    .unwrap_or_default(),
                session_affinity: kube_conf.and_then(|k| k.service.session_affinity.clone()),
                external_ips: kube_conf
                    .map(|k| k.service.external_ips.clone())
                    .unwrap_or_default(),
                load_balancer_source_ranges: Vec::new(),
            },
            storage: storage_config,
            image_pull_policy: image_pull_policy.unwrap_or_else(|| "IfNotPresent".to_string()),
            image_pull_secrets: kube_conf
                .map(|k| k.image_pull_secrets.clone())
                .unwrap_or_default(),
            cluster_domain: "cluster.local".to_string(),
        };

        // Apply advanced dynamic configurations directly to kube_config
        if let Some(ref configs) = dynamic_configs {
            crate::domain::config::dynamic::apply_to_kube_config(configs, &mut kube_config);
        }

        // Create cluster descriptor with kubeconfig options
        let descriptor = CurvineClusterDescriptor::new_with_config(
            namespace,
            cmd.kubeconfig.clone(),
            cmd.context.clone(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create cluster descriptor: {}", e))?;

        // Update cluster
        descriptor
            .update_cluster(&cluster_conf, &kube_config)
            .await
            .map_err(|e| {
                let error_msg = e.to_string();
                if error_msg.starts_with("Failed to update cluster: ") {
                    anyhow::anyhow!("{}", error_msg)
                } else {
                    anyhow::anyhow!("Cluster update failed: {}", error_msg)
                }
            })?;

        println!("Cluster {} updated successfully!", kube_config.cluster_id);
        Ok(())
    }
}

impl StatusCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        use crate::cli::display::TableRenderer;

        let cluster_id = self
            .cluster_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("cluster_id is required (use -c/--cluster-id)"))?;

        let descriptor = CurvineClusterDescriptor::new_with_config(
            self.namespace.clone(),
            self.kubeconfig.clone(),
            self.context.clone(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create cluster descriptor: {}", e))?;

        let status = descriptor
            .get_cluster_status(cluster_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get cluster status: {}", e))?;

        // Extract status information
        let master_name = status.master.as_ref().map(|m| m.name.as_str());
        let master_ready = status.master.as_ref().map(|m| m.ready_replicas).unwrap_or(0);
        let master_replicas = status.master.as_ref().map(|m| m.replicas).unwrap_or(0);

        let worker_name = status.worker.as_ref().map(|w| w.name.as_str());
        let worker_ready = status.worker.as_ref().map(|w| w.ready_replicas).unwrap_or(0);
        let worker_replicas = status.worker.as_ref().map(|w| w.replicas).unwrap_or(0);

        let service_name = status.service.as_ref().map(|s| s.name.as_str());
        let cluster_ip = status
            .service
            .as_ref()
            .and_then(|s| s.cluster_ip.as_deref());

        let configmap_name = status.configmap.as_ref().map(|c| c.name.as_str());

        // Render using new table renderer
        let renderer = TableRenderer::new();
        let output = renderer.render_cluster_status(
            &status.cluster_id,
            &self.namespace,
            master_name,
            master_ready,
            master_replicas,
            worker_name,
            worker_ready,
            worker_replicas,
            service_name,
            cluster_ip,
            configmap_name,
        );

        println!("{}", output);

        Ok(())
    }
}

impl DeleteCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        let cluster_id = self
            .cluster_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("cluster_id is required (use -c/--cluster-id)"))?;

        let descriptor = CurvineClusterDescriptor::new_with_config(
            self.namespace.clone(),
            self.kubeconfig.clone(),
            self.context.clone(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create cluster descriptor: {}", e))?;

        descriptor
            .delete_cluster(cluster_id, self.delete_pvcs)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete cluster: {}", e))?;

        println!("Cluster {} deleted successfully!", cluster_id);
        Ok(())
    }
}

impl ListCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        use crate::cli::display::{table::ClusterInfo, TableRenderer};

        let namespaces: Vec<String> = if let Some(ref ns) = self.namespace {
            vec![ns.clone()]
        } else {
            vec!["default".to_string(), "curvine".to_string()]
        };

        let mut all_clusters = Vec::new();

        for namespace in &namespaces {
            let descriptor = match CurvineClusterDescriptor::new(namespace.clone()).await {
                Ok(d) => d,
                Err(_) => continue,
            };

            match descriptor.list_clusters().await {
                Ok(clusters) => all_clusters.extend(clusters),
                Err(_) => continue,
            }
        }

        if all_clusters.is_empty() {
            println!(
                "No Curvine clusters found in namespace(s): {}",
                namespaces.join(", ")
            );
            return Ok(());
        }

        // Convert to ClusterInfo for rendering
        let cluster_infos: Vec<ClusterInfo> = all_clusters
            .iter()
            .map(|c| ClusterInfo {
                cluster_id: c.cluster_id.clone(),
                namespace: c.namespace.clone(),
                master_ready: c.master_ready,
                master_replicas: c.master_replicas,
                worker_ready: c.worker_ready,
                worker_replicas: c.worker_replicas,
            })
            .collect();

        // Render using new table renderer
        let renderer = TableRenderer::new();
        let output = renderer.render_clusters_list(&cluster_infos);
        println!("{}", output);

        Ok(())
    }
}

/// Parse dynamic configuration properties from -D key=value format
fn parse_dynamic_configs(configs: &[String]) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();

    for config in configs {
        let parts: Vec<&str> = config.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid config format: '{}'. Expected 'key=value'",
                config
            ));
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        if key.is_empty() {
            return Err(format!("Empty key in config: '{}'", config));
        }

        map.insert(key.to_string(), value.to_string());
    }

    Ok(map)
}
