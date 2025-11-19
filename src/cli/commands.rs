// CLI command definitions

use super::k8s::{DeleteCommand, DeployCommand, ListCommand, StatusCommand, UpdateCommand};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "curvine-kube",
    version,
    about = "Kubernetes deployment tool for Curvine cluster",
    long_about = "A standalone CLI tool for deploying and managing Curvine clusters on Kubernetes"
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Deploy a new Curvine cluster to Kubernetes (creates all resources)
    Deploy(DeployCommand),

    /// Update an existing Curvine cluster (modifies replicas, images, etc.)
    Update(UpdateCommand),

    /// List all Curvine clusters
    List(ListCommand),

    /// Show cluster status
    Status(StatusCommand),

    /// Delete a cluster
    Delete(DeleteCommand),
}
