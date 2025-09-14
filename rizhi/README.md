# Alert Aggregation System

A comprehensive alert aggregation and intelligent noise reduction system for the 5.1++ high-frequency arbitrage system.

## Features

### Core Functionality
- **Intelligent Alert Aggregation**: Groups related alerts using multiple similarity algorithms
- **Smart Suppression**: Reduces alert noise through pattern detection and machine learning
- **Root Cause Analysis**: Identifies probable root causes using Bayesian networks and expert rules
- **Comprehensive Audit Logging**: Full compliance tracking with GDPR support
- **Real-time Processing**: Sub-second alert processing with configurable intervals

### Advanced Features
- **Multi-dimensional Similarity Analysis**: Text, structural, temporal, and semantic similarity
- **Adaptive Thresholds**: Self-adjusting suppression thresholds based on alert patterns
- **Flapping Detection**: Automatic detection and suppression of oscillating alerts
- **Pattern Recognition**: Identifies alert storms, cascade patterns, and periodic behaviors
- **Compliance Framework**: Built-in support for regulatory compliance requirements

## Architecture

The system is composed of several interconnected modules:

```
alert_aggregation/
├── mod.rs                    # Main aggregation engine
├── similarity.rs             # Multi-algorithm similarity analysis
├── root_cause.rs            # Bayesian network root cause analysis
├── suppression.rs           # Intelligent alert suppression
├── audit.rs                 # Audit logging and compliance
├── alert_aggregation_config.toml  # Configuration file
└── README.md               # This file
```

## Configuration

The system is configured via `alert_aggregation_config.toml`. Key sections include:

### General Settings
```toml
[general]
max_active_alerts = 10000
alert_ttl_seconds = 3600
aggregation_interval_seconds = 30
enable_root_cause_analysis = true
enable_smart_suppression = true
default_similarity_threshold = 0.8
```

### Aggregation Rules
```toml
[[rules]]
id = "api_errors"
name = "API Error Aggregation"
window_seconds = 300
max_count = 10
action = "AggregateAndAlert"
enabled = true
```

### Suppression Rules
```toml
[[suppression_rules]]
id = "low_severity_flood"
name = "Low Severity Flood Protection"
enabled = true
duration_minutes = 30
```

## Usage

### Basic Example

```rust
use alert_aggregation::{AlertAggregationEngine, AlertAggregationConfig, RawAlert};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the engine
    let config = AlertAggregationConfig::default();
    let engine = AlertAggregationEngine::new(Some(config)).await?;
    
    // Create a raw alert
    let alert = RawAlert::new(
        "Database Connection Error".to_string(),
        "Connection timeout to primary database".to_string(),
        AlertSeverity::High,
        "database-service".to_string(),
    );
    
    // Process the alert
    let result = engine.process_alert(alert).await?;
    
    match result {
        AlertProcessingResult::Individual(alert) => {
            println!("Alert processed individually: {}", alert.id);
        },
        AlertProcessingResult::Aggregated(results) => {
            println!("Alert aggregated with {} results", results.len());
        },
        AlertProcessingResult::Suppressed(info) => {
            println!("Alert suppressed: {}", info.reason);
        },
        _ => {}
    }
    
    Ok(())
}
```

### Advanced Configuration

```rust
use alert_aggregation::*;

let config = AlertAggregationConfig {
    max_active_alerts: 50000,
    alert_ttl_seconds: 7200,
    aggregation_interval_seconds: 15,
    enable_root_cause_analysis: true,
    enable_smart_suppression: true,
    default_similarity_threshold: 0.85,
};

let engine = AlertAggregationEngine::new(Some(config)).await?;
```

## Similarity Analysis

The system employs multiple similarity algorithms:

### Text Similarity
- **Edit Distance**: Levenshtein distance with normalization
- **Cosine Similarity**: TF-IDF vectorization with cosine distance
- **Jaccard Similarity**: Token-based set similarity
- **Semantic Similarity**: Domain-specific word groupings

### Structural Similarity
- **Label Matching**: Key-value pair similarity with fuzzy matching
- **Metric Correlation**: Numerical value correlation analysis
- **Temporal Patterns**: Time-based relationship detection
- **Source Correlation**: Component and service relationship mapping

## Suppression Strategies

### Rule-Based Suppression
- Pattern matching on alert content
- Frequency-based thresholds
- Time window constraints
- Severity filtering

### Intelligent Suppression
- Flapping detection and mitigation
- Alert storm identification
- Cascade pattern recognition
- Adaptive threshold adjustment

### Smart Suppression
- Machine learning-based pattern detection
- Historical behavior analysis
- Correlation-based suppression
- Context-aware decision making

## Root Cause Analysis

The root cause analysis system combines multiple approaches:

### Expert Rules
- Domain-specific cause-effect relationships
- Pattern-based inference rules
- Confidence scoring and evidence collection

### Bayesian Networks
- Probabilistic inference models
- Conditional probability tables
- Posterior probability calculation

### Feature Extraction
- System component mapping
- Error keyword analysis
- Temporal pattern detection
- Metric trend analysis

## Audit and Compliance

### Audit Logging
- Complete event history tracking
- User action logging
- Configuration change tracking
- Export capabilities (JSON, CSV)

### Compliance Support
- GDPR compliance framework
- Configurable retention policies
- Data encryption support
- Access control integration

### Reporting
- Automated compliance reports
- Statistical analysis
- Trend identification
- Performance metrics

## Performance Characteristics

### Throughput
- **Target**: 10,000+ alerts per second
- **Latency**: Sub-500μs processing time
- **Memory**: Configurable working set limits
- **Storage**: Efficient data structures and caching

### Scalability
- Horizontal scaling support
- Distributed processing capability
- Load balancing integration
- Resource-aware throttling

## Monitoring and Observability

### Metrics
- Processing latency distribution
- Aggregation rule effectiveness
- Suppression statistics
- Root cause accuracy metrics

### Health Checks
- System component status
- Queue depth monitoring
- Memory usage tracking
- Error rate analysis

### Integration
- Prometheus metrics export
- Grafana dashboard templates
- OpenTelemetry tracing
- Structured logging

## Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run specific module tests
cargo test similarity
cargo test suppression
cargo test root_cause
cargo test audit
```

### Integration Tests
```bash
# Run integration tests
cargo test --test integration

# Run with logging
RUST_LOG=debug cargo test
```

### Performance Tests
```bash
# Run performance benchmarks
cargo bench

# Stress testing
cargo test stress_test --release
```

## Troubleshooting

### Common Issues

#### High Memory Usage
- Reduce `max_active_alerts` in configuration
- Decrease `alert_ttl_seconds`
- Enable alert cleanup job

#### Processing Delays
- Increase `aggregation_interval_seconds`
- Reduce similarity computation complexity
- Enable async processing

#### False Positives in Aggregation
- Adjust `default_similarity_threshold`
- Fine-tune aggregation rules
- Review similarity algorithm weights

### Debug Mode
Enable debug logging:
```toml
[logging]
level = "DEBUG"
structured_logging = true
include_trace_id = true
```

### Performance Profiling
```bash
# CPU profiling
cargo flamegraph --test integration

# Memory profiling
cargo valgrind --test integration
```

## Contributing

### Development Setup
1. Install Rust toolchain
2. Clone repository
3. Run tests: `cargo test`
4. Check formatting: `cargo fmt --check`
5. Run linter: `cargo clippy`

### Code Style
- Follow Rust naming conventions
- Use structured error handling
- Include comprehensive tests
- Document public APIs
- Add performance benchmarks

### Pull Request Process
1. Create feature branch
2. Implement changes with tests
3. Update documentation
4. Run full test suite
5. Submit pull request

## License

This project is part of the 5.1++ high-frequency arbitrage system and is subject to the project's license terms.

## Support

For technical support and questions:
- Create GitHub issue for bugs
- Use discussions for questions
- Contact development team for urgent issues

## Changelog

### Version 1.0.0
- Initial release with core aggregation functionality
- Multi-algorithm similarity analysis
- Basic suppression rules
- Audit logging framework

### Version 1.1.0
- Added root cause analysis
- Implemented smart suppression
- Enhanced performance optimization
- Extended configuration options

### Version 1.2.0 (Current)
- Machine learning integration
- Advanced pattern detection
- Compliance framework enhancement
- Real-time processing optimization