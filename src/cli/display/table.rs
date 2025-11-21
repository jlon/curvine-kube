//! Table rendering for CLI output

use super::{ColorTheme, StatusIcon};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement, Table};

/// Cluster information for list display
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    pub cluster_id: String,
    pub namespace: String,
    pub master_ready: u32,
    pub master_replicas: u32,
    pub worker_ready: u32,
    pub worker_replicas: u32,
}

/// Table renderer for formatted output
pub struct TableRenderer {
    theme: ColorTheme,
}

impl Default for TableRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TableRenderer {
    /// Create a new table renderer with default theme
    pub fn new() -> Self {
        Self {
            theme: ColorTheme::default(),
        }
    }

    /// Render clusters list as a formatted table
    pub fn render_clusters_list(&self, clusters: &[ClusterInfo]) -> String {
        if clusters.is_empty() {
            return "No Curvine clusters found".to_string();
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("CLUSTER").set_alignment(CellAlignment::Left),
                Cell::new("NAMESPACE").set_alignment(CellAlignment::Left),
                Cell::new("MASTER").set_alignment(CellAlignment::Center),
                Cell::new("WORKER").set_alignment(CellAlignment::Center),
                Cell::new("STATUS").set_alignment(CellAlignment::Center),
            ]);

        for cluster in clusters {
            let master_icon = StatusIcon::get_replica_icon(
                cluster.master_ready,
                cluster.master_replicas,
            );
            let worker_icon = StatusIcon::get_replica_icon(
                cluster.worker_ready,
                cluster.worker_replicas,
            );

            // Determine overall status
            let overall_status = if cluster.master_ready == cluster.master_replicas
                && cluster.worker_ready == cluster.worker_replicas
            {
                StatusIcon::get_status_text(cluster.master_ready, cluster.master_replicas)
            } else if cluster.master_ready > 0 || cluster.worker_ready > 0 {
                "Degraded"
            } else {
                "Failed"
            };

            let status_icon = if overall_status == "Running" {
                StatusIcon::SUCCESS
            } else if overall_status == "Degraded" {
                StatusIcon::WARNING
            } else {
                StatusIcon::ERROR
            };

            // Determine colors
            let master_color = self.theme.get_replica_color(
                cluster.master_ready,
                cluster.master_replicas,
            );
            let worker_color = self.theme.get_replica_color(
                cluster.worker_ready,
                cluster.worker_replicas,
            );
            let status_color = if overall_status == "Running" {
                Color::Green
            } else if overall_status == "Degraded" {
                Color::Yellow
            } else {
                Color::Red
            };

            table.add_row(vec![
                Cell::new(&cluster.cluster_id),
                Cell::new(&cluster.namespace),
                Cell::new(format!(
                    "{} {}/{}",
                    master_icon, cluster.master_ready, cluster.master_replicas
                ))
                .fg(master_color),
                Cell::new(format!(
                    "{} {}/{}",
                    worker_icon, cluster.worker_ready, cluster.worker_replicas
                ))
                .fg(worker_color),
                Cell::new(format!("{} {}", status_icon, overall_status)).fg(status_color),
            ]);
        }

        let mut output = String::new();
        output.push_str(&format!(
            "╭─ Curvine Clusters {} ─╮\n",
            format!("[{} clusters]", clusters.len())
                .bright_black()
                .to_string()
        ));
        output.push_str(&table.to_string());
        output.push('\n');
        output.push_str(&format!(
            "Legend: {} Healthy  {} Degraded  {} Failed\n",
            StatusIcon::SUCCESS.green(),
            StatusIcon::WARNING.yellow(),
            StatusIcon::ERROR.red()
        ));

        output
    }

    /// Render cluster status in a compact three-column format
    pub fn render_cluster_status(
        &self,
        cluster_id: &str,
        namespace: &str,
        master_name: Option<&str>,
        master_ready: u32,
        master_replicas: u32,
        worker_name: Option<&str>,
        worker_ready: u32,
        worker_replicas: u32,
        service_name: Option<&str>,
        cluster_ip: Option<&str>,
        configmap_name: Option<&str>,
    ) -> String {
        // Calculate overall status
        let overall_status = if master_ready == master_replicas && worker_ready == worker_replicas
        {
            format!("{} Running", StatusIcon::SUCCESS)
        } else if master_ready > 0 || worker_ready > 0 {
            format!("{} Degraded", StatusIcon::WARNING)
        } else {
            format!("{} Failed", StatusIcon::ERROR)
        };

        let overall_status_color = if master_ready == master_replicas && worker_ready == worker_replicas {
            Color::Green
        } else if master_ready > 0 || worker_ready > 0 {
            Color::Yellow
        } else {
            Color::Red
        };

        // Master status
        let master_icon = StatusIcon::get_replica_icon(master_ready, master_replicas);
        let master_color = self.theme.get_replica_color(master_ready, master_replicas);
        let master_status = format!("{} {}/{}", master_icon, master_ready, master_replicas);

        // Worker status
        let worker_icon = StatusIcon::get_replica_icon(worker_ready, worker_replicas);
        let worker_color = self.theme.get_replica_color(worker_ready, worker_replicas);
        let worker_status = format!("{} {}/{}", worker_icon, worker_ready, worker_replicas);

        // Create a three-column table
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        // Header
        table.add_row(vec![
            Cell::new(format!("📊 Curvine Cluster Status")).set_alignment(CellAlignment::Center)
        ]);

        // Basic info
        table.add_row(vec![Cell::new("")]);
        table.add_row(vec![
            Cell::new(format!("Cluster: {} | Namespace: {}", cluster_id, namespace))
                .set_alignment(CellAlignment::Left)
        ]);
        table.add_row(vec![
            Cell::new(format!("Status: {}", overall_status))
                .fg(overall_status_color)
                .set_alignment(CellAlignment::Left)
        ]);

        // Three columns header
        table.add_row(vec![Cell::new("")]);
        
        // Create three-column layout using a single row with formatted text
        let master_text = if let Some(name) = master_name {
            format!("🔷 Master Nodes\n  StatefulSet: {}\n  Replicas: {}", name, master_status)
        } else {
            "🔷 Master Nodes\n  StatefulSet not found".to_string()
        };

        let worker_text = if let Some(name) = worker_name {
            format!("🔶 Worker Nodes\n  StatefulSet: {}\n  Replicas: {}", name, worker_status)
        } else {
            "🔶 Worker Nodes\n  StatefulSet not found".to_string()
        };

        let service_text = if let Some(name) = service_name {
            let mut text = format!("🌐 Services\n  Service: {}", name);
            if let Some(ip) = cluster_ip {
                text.push_str(&format!("\n  Cluster IP: {}", ip));
            }
            if let Some(cm) = configmap_name {
                text.push_str(&format!("\n  ConfigMap: {}", cm));
            }
            text
        } else {
            "🌐 Services\n  Service not found".to_string()
        };

        // Add the three columns as separate cells in one row
        table.add_row(vec![
            Cell::new(master_text)
                .fg(master_color)
                .set_alignment(CellAlignment::Left),
            Cell::new(worker_text)
                .fg(worker_color)
                .set_alignment(CellAlignment::Left),
            Cell::new(service_text)
                .fg(Color::Cyan)
                .set_alignment(CellAlignment::Left),
        ]);

        table.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_clusters() {
        let renderer = TableRenderer::new();
        let output = renderer.render_clusters_list(&[]);
        assert!(output.contains("No Curvine clusters found"));
    }

    #[test]
    fn test_render_single_cluster() {
        let renderer = TableRenderer::new();
        let clusters = vec![ClusterInfo {
            cluster_id: "test-cluster".to_string(),
            namespace: "default".to_string(),
            master_ready: 3,
            master_replicas: 3,
            worker_ready: 5,
            worker_replicas: 5,
        }];

        let output = renderer.render_clusters_list(&clusters);
        assert!(output.contains("test-cluster"));
        assert!(output.contains("default"));
        assert!(output.contains("3/3"));
        assert!(output.contains("5/5"));
    }
}
