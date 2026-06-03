# Changelog

## 0.1.0 (2026-05-28)

### Filters
- Tremor rejection filter: discards micro movements below configurable pixel threshold.
- One Euro smoothing filter: speed adaptive low pass filter for pointer position.
- Dwell click detector: fires a left click when the pointer holds still within a radius for a configurable duration.
- Click zone snapping: snaps pointer to defined target centers when entering a zone radius.
- Filter pipeline chaining tremor, smoothing, click zone snap, and dwell detection.
- Pipeline statistics tracker: counts events, suppressions, dwell clicks, and snap activations.

### Platform
- Windows backend using WH_MOUSE_LL hook with suppress reinject guard loop.
- Linux and macOS backends scaffolded with documented approach and trait implementation.

### Configuration
- TOML configuration with serde serialization/deserialization and per field defaults.
- Config validation: rejects negative thresholds, zero cutoff frequencies, invalid zone radii.
- `status` command to check config file location and validity.
- `validate` command to check a config file without starting the daemon.

### CLI
- `start` subcommand to run the daemon with optional config path.
- `defaults` subcommand to print the default config to stdout.
- `status` subcommand to show config location and parse status.
- `validate` subcommand to check a config file.

### Testing
- Unit and integration tests for all filters, config parsing, validation, pipeline stats, and edge cases.
- CI with GitHub Actions matrix (ubuntu for fmt/clippy/core tests, windows for full build).
