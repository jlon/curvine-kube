//! Status icons for CLI output

/// Status icons for different states
pub struct StatusIcon;

impl StatusIcon {
    /// Success icon (all replicas ready)
    pub const SUCCESS: &'static str = "✓";
    
    /// Warning icon (partial replicas ready)
    pub const WARNING: &'static str = "⚠";
    
    /// Error icon (no replicas ready)
    pub const ERROR: &'static str = "✗";
    
    /// Pending icon (waiting)
    pub const PENDING: &'static str = "⏳";
    
    /// Unknown icon
    pub const UNKNOWN: &'static str = "?";
    
    /// Get status icon based on ready/total replicas
    pub fn get_replica_icon(ready: u32, total: u32) -> &'static str {
        if total == 0 {
            Self::UNKNOWN
        } else if ready == total {
            Self::SUCCESS
        } else if ready > 0 {
            Self::WARNING
        } else {
            Self::ERROR
        }
    }
    
    /// Get status text based on ready/total replicas
    pub fn get_status_text(ready: u32, total: u32) -> &'static str {
        if total == 0 {
            "Unknown"
        } else if ready == total {
            "Running"
        } else if ready > 0 {
            "Degraded"
        } else {
            "Failed"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_replica_icon() {
        assert_eq!(StatusIcon::get_replica_icon(3, 3), StatusIcon::SUCCESS);
        assert_eq!(StatusIcon::get_replica_icon(2, 3), StatusIcon::WARNING);
        assert_eq!(StatusIcon::get_replica_icon(0, 3), StatusIcon::ERROR);
        assert_eq!(StatusIcon::get_replica_icon(0, 0), StatusIcon::UNKNOWN);
    }

    #[test]
    fn test_get_status_text() {
        assert_eq!(StatusIcon::get_status_text(3, 3), "Running");
        assert_eq!(StatusIcon::get_status_text(2, 3), "Degraded");
        assert_eq!(StatusIcon::get_status_text(0, 3), "Failed");
        assert_eq!(StatusIcon::get_status_text(0, 0), "Unknown");
    }
}
