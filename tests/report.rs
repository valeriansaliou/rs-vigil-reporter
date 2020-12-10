extern crate env_logger;
extern crate log;
extern crate vigil_reporter;

use std::thread;
use std::time::Duration;

use env_logger::Builder;
use log::LevelFilter;
use vigil_reporter::Reporter;

fn setup() {
    Builder::new()
        .filter(None, LevelFilter::Trace)
        .try_init()
        .ok();
}

#[test]
fn initialize_valid() {
    setup();

    Reporter::new("http://status.example.com.local", "YOUR_TOKEN_SECRET")
        .probe_id("relay")
        .node_id("socket-client")
        .replica_id("192.168.1.10")
        .interval(Duration::from_secs(30))
        .build();
}

#[test]
#[should_panic]
fn initialize_invalid_probe_id() {
    setup();

    Reporter::new("http://status.example.com.local", "YOUR_TOKEN_SECRET")
        .node_id("socket-client")
        .replica_id("192.168.1.10")
        .build();
}

#[test]
#[should_panic]
fn initialize_invalid_node_id() {
    setup();

    Reporter::new("http://status.example.com.local", "YOUR_TOKEN_SECRET")
        .probe_id("relay")
        .replica_id("192.168.1.10")
        .build();
}

#[test]
#[should_panic]
fn initialize_invalid_replica_id() {
    setup();

    Reporter::new("http://status.example.com.local", "YOUR_TOKEN_SECRET")
        .probe_id("relay")
        .node_id("socket-client")
        .build();
}

#[test]
fn run_and_end_valid() {
    setup();

    let reporter = Reporter::new("http://status.example.com.local", "YOUR_TOKEN_SECRET")
        .probe_id("relay")
        .node_id("socket-client")
        .replica_id("192.168.1.10")
        .build();

    assert_eq!(reporter.run().is_ok(), true);

    // Hold on while the first reporting request fires (in the wild)
    thread::sleep(Duration::from_secs(15));
}
