# rs-vigil-reporter

[![Test and Build](https://github.com/valeriansaliou/rs-vigil-reporter/workflows/Test%20and%20Build/badge.svg?branch=master)](https://github.com/valeriansaliou/rs-vigil-reporter/actions?query=workflow%3A%22Test+and+Build%22) [![Build and Release](https://github.com/valeriansaliou/rs-vigil-reporter/workflows/Build%20and%20Release/badge.svg)](https://github.com/valeriansaliou/rs-vigil-reporter/actions?query=workflow%3A%22Build+and+Release%22) [![Buy Me A Coffee](https://img.shields.io/badge/buy%20me%20a%20coffee-donate-yellow.svg)](https://www.buymeacoffee.com/valeriansaliou)

* [Documentation](https://docs.rs/crate/vigil-reporter)
* [Crate](https://crates.io/crates/vigil-reporter)

**Vigil Reporter for Rust. Used in pair with Vigil, the Microservices Status Page.**

Vigil Reporter is used to actively submit health information to Vigil from your apps. Apps are best monitored via application probes, which are able to report detailed system information such as CPU and RAM load. This lets Vigil show if an application host system is under high load.

**🇭🇺 Crafted in Budapest, Hungary.**

## Who uses it?

_👋 You use vigil-reporter and you want to be listed there? [Contact me](https://valeriansaliou.name/)._

## How to install?

Include `vigil-reporter` in your `Cargo.toml` dependencies:

```toml
[dependencies]
vigil-reporter = "1.0"
```

## How to use?

### Create reporter

`vigil-reporter` can be instantiated as such:

```rust
extern crate vigil_reporter;

use std::time::Duration;
use vigil_reporter::Reporter;

// Build reporter
// `page_url` + `reporter_token` from Vigil `config.cfg`
let reporter = Reporter::new("https://status.example.com", "YOUR_TOKEN_SECRET")
  .probe_id("relay")                  // Probe ID containing the parent Node for Replica
  .node_id("socket-client")           // Node ID containing Replica
  .replica_id("192.168.1.10")         // Unique Replica ID for instance (ie. your IP on the LAN)
  .interval(Duration::from_secs(30))  // Reporting interval (in seconds; defaults to 30 seconds if not set)
  .build();

// Run reporter (starts reporting)
reporter.run();
```

## What is Vigil?

ℹ️ **Wondering what Vigil is?** Check out **[valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)**.
