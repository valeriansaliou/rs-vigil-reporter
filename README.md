# rs-vigil-reporter

[![Build Status](https://img.shields.io/travis/valeriansaliou/rs-vigil-reporter/master.svg)](https://travis-ci.org/valeriansaliou/rs-vigil-reporter)

* [Documentation](https://docs.rs/crate/vigil-reporter)
* [Crate](https://crates.io/crates/vigil-reporter)

**Vigil Reporter for Rust. Used in pair with Vigil, the Microservices Status Page.**

Vigil Reporter is used to actively submit health information to Vigil from your apps. Apps are best monitored via application probes, which are able to report detailed system information such as CPU and RAM load. This lets Vigil show if an application host system is under high load.

## How to install?

Include `vigil-reporter` in your `Cargo.toml` dependencies:

```toml
[dependencies]
vigil-reporter = "1.0"
```

## How to use?

### 1. Create reporter

`vigil-reporter` can be instantiated as such:

```rust
extern crate vigil_reporter;

use vigil_reporter::Reporter;

// Build reporter
// `page_url` + `reporter_token` from Vigil `config.cfg`
let reporter = Reporter::new("https://status.example.com", "YOUR_TOKEN_SECRET")
  .probe_id("relay")                  // Probe ID containing the parent Node for Replica
  .node_id("socket-client")           // Node ID containing Replica
  .replica_id("192.168.1.10")         // Unique Replica ID for instance (ie. your IP on the LAN)
  .interval(Duration::from_secs(30))  // Reporting interval (in seconds; defaults to 30 seconds if not set)
  .debug(true)                        // Whether to debug or not
  .build();

// Run reporter (starts reporting)
reporter.run();
```

### 2. Teardown reporter

If you need to teardown an active reporter, you can use the `end()` method to unbind it.

```javascript
// End reporter (stops reporting)
reporter.end();
```

## What is Vigil?

ℹ️ **Wondering what Vigil is?** Check out **[valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)**.
