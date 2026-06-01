/// TOML configuration with serde deserialization and validation.
/// Every field has a sensible default so the config file can be
/// partial or missing entirely.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub tremor: TremorConfig,
    pub smoothing: SmoothingConfig,
    pub dwell: DwellConfig,
    pub clickzones: ClickZonesConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TremorConfig {
    pub enabled: bool,
    /// Minimum pixel distance for a move to be accepted.
    pub threshold: f64,
}

impl Default for TremorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 3.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SmoothingConfig {
    pub enabled: bool,
    /// Minimum cutoff frequency (Hz) for the One Euro filter.
    /// Lower values produce more smoothing at rest.
    pub min_cutoff: f64,
    /// Speed coefficient. Higher values reduce smoothing during fast movement.
    pub beta: f64,
}

impl Default for SmoothingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_cutoff: 1.0,
            beta: 0.007,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DwellConfig {
    pub enabled: bool,
    /// Dwell time in milliseconds before a click fires.
    pub time_ms: f64,
    /// Maximum pixel radius for position to count as "still".
    pub radius: f64,
}

impl Default for DwellConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            time_ms: 800.0,
            radius: 10.0,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ClickZonesConfig {
    pub enabled: bool,
    pub zones: Vec<ZoneEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZoneEntry {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

impl Config {
    /// Load config from a specific path.
    pub fn load_from(path: &std::path::Path) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read config file: {}", e))?;
        let config: Self =
            toml::from_str(&contents).map_err(|e| format!("failed to parse config: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    /// Load config from the default platform location, falling back to
    /// defaults if the file does not exist.
    pub fn load_default() -> Self {
        match Self::default_path() {
            Some(path) if path.exists() => Self::load_from(&path).unwrap_or_else(|e| {
                eprintln!("warning: {}, using defaults", e);
                Self::default()
            }),
            _ => Self::default(),
        }
    }

    /// Default config file path for the current platform.
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("steady").join("config.toml"))
    }

    /// Check whether a config file exists at the default path.
    pub fn default_exists() -> bool {
        Self::default_path().map(|p| p.exists()).unwrap_or(false)
    }

    /// Validate all parameter ranges. Returns Ok(()) if valid, or an
    /// error string describing the first problem found.
    pub fn validate(&self) -> Result<(), String> {
        if self.tremor.threshold < 0.0 {
            return Err("tremor.threshold must be >= 0".to_string());
        }
        if self.smoothing.min_cutoff <= 0.0 {
            return Err("smoothing.min_cutoff must be > 0".to_string());
        }
        if self.smoothing.beta < 0.0 {
            return Err("smoothing.beta must be >= 0".to_string());
        }
        if self.dwell.time_ms <= 0.0 {
            return Err("dwell.time_ms must be > 0".to_string());
        }
        if self.dwell.radius <= 0.0 {
            return Err("dwell.radius must be > 0".to_string());
        }
        for (i, zone) in self.clickzones.zones.iter().enumerate() {
            if zone.radius <= 0.0 {
                return Err(format!("clickzones.zones[{}].radius must be > 0", i));
            }
        }
        Ok(())
    }

    /// Count how many filters are currently enabled.
    pub fn enabled_filter_count(&self) -> usize {
        let mut count = 0;
        if self.tremor.enabled {
            count += 1;
        }
        if self.smoothing.enabled {
            count += 1;
        }
        if self.dwell.enabled {
            count += 1;
        }
        if self.clickzones.enabled {
            count += 1;
        }
        count
    }

    /// Serialize the default config to TOML for printing.
    pub fn default_toml() -> String {
        r#"# steady configuration

[tremor]
enabled = true
# Minimum pixel distance for a move to be accepted.
threshold = 3.0

[smoothing]
enabled = true
# Minimum cutoff frequency (Hz). Lower = more smoothing at rest.
min_cutoff = 1.0
# Speed coefficient. Higher = less smoothing during fast movement.
beta = 0.007

[dwell]
enabled = false
# Dwell time in milliseconds before a click fires.
time_ms = 800.0
# Maximum pixel radius for position to count as still.
radius = 10.0

[clickzones]
enabled = false
# Define zones as [[clickzones.zones]]
# [[clickzones.zones]]
# x = 960.0
# y = 540.0
# radius = 30.0
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_values() {
        let c = Config::default();
        assert!(c.tremor.enabled);
        assert!(c.smoothing.enabled);
        assert!(!c.dwell.enabled);
        assert!(!c.clickzones.enabled);
        assert!(c.tremor.threshold > 0.0);
        assert!(c.smoothing.min_cutoff > 0.0);
    }

    #[test]
    fn parse_partial_toml() {
        let toml_str = r#"
[tremor]
threshold = 5.0
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.tremor.threshold, 5.0);
        // Other fields should have defaults
        assert!(config.tremor.enabled);
        assert!(config.smoothing.enabled);
    }

    #[test]
    fn parse_full_toml() {
        let toml_str = r#"
[tremor]
enabled = false
threshold = 2.0

[smoothing]
enabled = true
min_cutoff = 0.5
beta = 0.01

[dwell]
enabled = true
time_ms = 1000.0
radius = 15.0

[clickzones]
enabled = true

[[clickzones.zones]]
x = 100.0
y = 200.0
radius = 25.0
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.tremor.enabled);
        assert_eq!(config.tremor.threshold, 2.0);
        assert!(config.dwell.enabled);
        assert_eq!(config.dwell.time_ms, 1000.0);
        assert_eq!(config.clickzones.zones.len(), 1);
        assert_eq!(config.clickzones.zones[0].x, 100.0);
    }

    #[test]
    fn parse_empty_toml() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.tremor.enabled);
        assert_eq!(config.tremor.threshold, 3.0);
    }

    #[test]
    fn validate_default_config_passes() {
        let c = Config::default();
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validate_negative_tremor_threshold_fails() {
        let mut c = Config::default();
        c.tremor.threshold = -1.0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_zero_min_cutoff_fails() {
        let mut c = Config::default();
        c.smoothing.min_cutoff = 0.0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_negative_beta_fails() {
        let mut c = Config::default();
        c.smoothing.beta = -0.5;
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_zero_dwell_time_fails() {
        let mut c = Config::default();
        c.dwell.time_ms = 0.0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_zero_zone_radius_fails() {
        let mut c = Config::default();
        c.clickzones.zones.push(ZoneEntry {
            x: 100.0,
            y: 100.0,
            radius: 0.0,
        });
        assert!(c.validate().is_err());
    }

    #[test]
    fn enabled_filter_count_works() {
        let c = Config::default();
        // Default: tremor + smoothing enabled, dwell + clickzones disabled
        assert_eq!(c.enabled_filter_count(), 2);
    }
}
