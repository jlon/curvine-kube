//! Color theme for CLI output

use comfy_table::Color as TableColor;

/// Color theme for terminal output
#[derive(Debug, Clone)]
pub struct ColorTheme {
    pub success: TableColor,
    pub warning: TableColor,
    pub error: TableColor,
    pub info: TableColor,
    pub muted: TableColor,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            success: TableColor::Green,
            warning: TableColor::Yellow,
            error: TableColor::Red,
            info: TableColor::Cyan,
            muted: TableColor::DarkGrey,
        }
    }
}

impl ColorTheme {
    /// Get color based on replica status
    pub fn get_replica_color(&self, ready: u32, total: u32) -> TableColor {
        if total == 0 {
            self.muted
        } else if ready == total {
            self.success
        } else if ready > 0 {
            self.warning
        } else {
            self.error
        }
    }
}

/// Convert comfy_table::Color to colored::Color string representation
pub fn table_color_to_colored_str(color: TableColor) -> &'static str {
    match color {
        TableColor::Green => "green",
        TableColor::Yellow => "yellow",
        TableColor::Red => "red",
        TableColor::Cyan => "cyan",
        TableColor::DarkGrey => "bright black",
        _ => "white",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = ColorTheme::default();
        assert_eq!(theme.success, TableColor::Green);
        assert_eq!(theme.warning, TableColor::Yellow);
        assert_eq!(theme.error, TableColor::Red);
    }

    #[test]
    fn test_get_replica_color() {
        let theme = ColorTheme::default();
        assert_eq!(theme.get_replica_color(3, 3), TableColor::Green);
        assert_eq!(theme.get_replica_color(2, 3), TableColor::Yellow);
        assert_eq!(theme.get_replica_color(0, 3), TableColor::Red);
        assert_eq!(theme.get_replica_color(0, 0), TableColor::DarkGrey);
    }
}
