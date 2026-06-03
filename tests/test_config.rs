use steady::config::Config;

#[test]
fn default_config_values() {
    let c = Config::default();
    assert!(c.tremor.enabled);
    assert_eq!(c.tremor.threshold, 3.0);
    assert!(c.smoothing.enabled);
    assert_eq!(c.smoothing.min_cutoff, 1.0);
    assert_eq!(c.smoothing.beta, 0.007);
    assert!(!c.dwell.enabled);
    assert_eq!(c.dwell.time_ms, 800.0);
    assert_eq!(c.dwell.radius, 10.0);
    assert!(!c.clickzones.enabled);
    assert!(c.clickzones.zones.is_empty());
}

#[test]
fn empty_toml_gives_defaults() {
    let c: Config = toml::from_str("").unwrap();
    assert!(c.tremor.enabled);
    assert_eq!(c.tremor.threshold, 3.0);
    assert!(c.smoothing.enabled);
}

#[test]
fn partial_toml_fills_defaults() {
    let toml_str = r#"
[tremor]
threshold = 7.5
"#;
    let c: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(c.tremor.threshold, 7.5);
    assert!(c.tremor.enabled); // default
    assert!(c.smoothing.enabled); // default
    assert_eq!(c.smoothing.min_cutoff, 1.0); // default
}

#[test]
fn full_config_parses() {
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
time_ms = 1200.0
radius = 15.0

[clickzones]
enabled = true

[[clickzones.zones]]
x = 960.0
y = 540.0
radius = 25.0

[[clickzones.zones]]
x = 100.0
y = 100.0
radius = 40.0
"#;
    let c: Config = toml::from_str(toml_str).unwrap();
    assert!(!c.tremor.enabled);
    assert_eq!(c.tremor.threshold, 2.0);
    assert!(c.smoothing.enabled);
    assert_eq!(c.smoothing.min_cutoff, 0.5);
    assert_eq!(c.smoothing.beta, 0.01);
    assert!(c.dwell.enabled);
    assert_eq!(c.dwell.time_ms, 1200.0);
    assert_eq!(c.dwell.radius, 15.0);
    assert!(c.clickzones.enabled);
    assert_eq!(c.clickzones.zones.len(), 2);
    assert_eq!(c.clickzones.zones[0].x, 960.0);
    assert_eq!(c.clickzones.zones[1].radius, 40.0);
}

#[test]
fn only_dwell_section() {
    let toml_str = r#"
[dwell]
enabled = true
time_ms = 500.0
"#;
    let c: Config = toml::from_str(toml_str).unwrap();
    assert!(c.dwell.enabled);
    assert_eq!(c.dwell.time_ms, 500.0);
    assert_eq!(c.dwell.radius, 10.0); // default
                                      // Other sections should be defaults
    assert!(c.tremor.enabled);
    assert!(!c.clickzones.enabled);
}

#[test]
fn missing_file_gives_error() {
    let path = std::path::Path::new("/nonexistent/path/config.toml");
    let result = Config::load_from(path);
    assert!(result.is_err());
}

#[test]
fn default_toml_output_parses() {
    let toml_str = Config::default_toml();
    let c: Config = toml::from_str(&toml_str).unwrap();
    assert!(c.tremor.enabled);
    assert!(c.smoothing.enabled);
    assert!(!c.dwell.enabled);
}

#[test]
fn validation_accepts_defaults() {
    let c = Config::default();
    assert!(c.validate().is_ok());
}

#[test]
fn validation_rejects_negative_threshold() {
    let mut c = Config::default();
    c.tremor.threshold = -5.0;
    assert!(c.validate().is_err());
}

#[test]
fn validation_rejects_zero_min_cutoff() {
    let mut c = Config::default();
    c.smoothing.min_cutoff = 0.0;
    let err = c.validate().unwrap_err();
    assert!(err.contains("min_cutoff"));
}

#[test]
fn validation_rejects_negative_beta() {
    let mut c = Config::default();
    c.smoothing.beta = -0.1;
    let err = c.validate().unwrap_err();
    assert!(err.contains("beta"));
}

#[test]
fn validation_rejects_zero_dwell_time() {
    let mut c = Config::default();
    c.dwell.time_ms = 0.0;
    let err = c.validate().unwrap_err();
    assert!(err.contains("time_ms"));
}

#[test]
fn validation_rejects_zero_zone_radius() {
    let mut c = Config::default();
    c.clickzones.zones.push(steady::config::ZoneEntry {
        x: 0.0,
        y: 0.0,
        radius: 0.0,
    });
    let err = c.validate().unwrap_err();
    assert!(err.contains("radius"));
}

#[test]
fn enabled_filter_count_all_disabled() {
    let mut c = Config::default();
    c.tremor.enabled = false;
    c.smoothing.enabled = false;
    c.dwell.enabled = false;
    c.clickzones.enabled = false;
    assert_eq!(c.enabled_filter_count(), 0);
}

#[test]
fn enabled_filter_count_all_enabled() {
    let mut c = Config::default();
    c.tremor.enabled = true;
    c.smoothing.enabled = true;
    c.dwell.enabled = true;
    c.clickzones.enabled = true;
    assert_eq!(c.enabled_filter_count(), 4);
}

#[test]
fn multiple_zones_parse() {
    let toml_str = r#"
[clickzones]
enabled = true

[[clickzones.zones]]
x = 100.0
y = 100.0
radius = 20.0

[[clickzones.zones]]
x = 200.0
y = 300.0
radius = 50.0

[[clickzones.zones]]
x = 960.0
y = 540.0
radius = 30.0
"#;
    let c: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(c.clickzones.zones.len(), 3);
    assert_eq!(c.clickzones.zones[2].x, 960.0);
}

#[test]
fn zero_threshold_is_valid() {
    let mut c = Config::default();
    c.tremor.threshold = 0.0;
    assert!(c.validate().is_ok());
}

#[test]
fn default_path_returns_something() {
    // On any standard system, config_dir should return a path
    let path = Config::default_path();
    assert!(path.is_some());
}
