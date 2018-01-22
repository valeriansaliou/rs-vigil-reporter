//! rs-vigil-reporter Vigil Reporter for Rust. Used in pair with Vigil, the Microservices Status Page.

#[macro_use]
extern crate log;

use std::thread;
use std::time::Duration;

static LOG_NAME: &'static str = "Vigil Reporter";

pub struct Reporter<'a> {
    url: &'a str,
    token: &'a str,
    probe_id: Option<&'a str>,
    node_id: Option<&'a str>,
    replica_id: Option<&'a str>,
    interval: Duration,
}

pub struct ReporterBuilder<'a> {
    reporter: Reporter<'a>,
}

struct ReporterManager {
    url: String,
    token: String,
    probe_id: String,
    node_id: String,
    replica_id: String,
    interval: Duration,
}

impl <'a>Reporter<'a> {
    pub fn new(url: &'a str, token: &'a str) -> ReporterBuilder<'a> {
        ReporterBuilder {
            reporter: Reporter {
                url: url,
                token: token,
                probe_id: None,
                node_id: None,
                replica_id: None,
                interval: Duration::from_secs(30),
            }
        }
    }

    pub fn run(&self) -> Result<(), ()> {
        debug!("{}: Will run using URL: {}", LOG_NAME, self.url);

        // Build thread manager context?
        match (self.probe_id, self.node_id, self.replica_id) {
            (Some(probe_id), Some(node_id), Some(replica_id)) => {
                let manager = ReporterManager {
                    url: self.url.to_owned(),
                    token: self.token.to_owned(),
                    probe_id: probe_id.to_owned(),
                    node_id: node_id.to_owned(),
                    replica_id: replica_id.to_owned(),
                    interval: self.interval,
                };

                // Spawn thread
                thread::spawn(move || manager.run());

                Ok(())
            },
            _ => Err(()),
        }
    }

    pub fn end(&self) -> Result<(), ()> {
        debug!("{}: Will end", LOG_NAME);

        // TODO: use channels to stop the thread
        // @see: https://stackoverflow.com/questions/26199926/how-to-terminate-or-suspend-a-rust-thread-from-another-thread

        // TODO
        Err(())
    }
}

impl <'a>ReporterBuilder<'a> {
    pub fn build(self) -> Reporter<'a> {
        if self.reporter.probe_id.is_none() {
            panic!("missing probe_id");
        }
        if self.reporter.node_id.is_none() {
            panic!("missing node_id");
        }
        if self.reporter.replica_id.is_none() {
            panic!("missing replica_id");
        }

        self.reporter
    }

    pub fn probe_id(mut self, probe_id: &'a str) -> ReporterBuilder<'a> {
        self.reporter.probe_id = Some(probe_id);

        self
    }

    pub fn node_id(mut self, node_id: &'a str) -> ReporterBuilder<'a> {
        self.reporter.node_id = Some(node_id);

        self
    }

    pub fn replica_id(mut self, replica_id: &'a str) -> ReporterBuilder<'a> {
        self.reporter.replica_id = Some(replica_id);

        self
    }

    pub fn interval(mut self, interval: Duration) -> ReporterBuilder<'a> {
        self.reporter.interval = interval;

        self
    }
}

impl ReporterManager {
    pub fn run(&self) {
        debug!("{}: Now running", LOG_NAME);

        // Schedule first report after 10 seconds
        thread::sleep(Duration::from_secs(10));

        loop {
            self.report();

            thread::sleep(self.interval);
        }

        debug!("{}: Now ended", LOG_NAME);
    }

    fn report(&self) {
        debug!("{}: Will dispatch request", LOG_NAME);

        // TODO: be fault tolerant
        // TODO: enforce timeout
        // TODO: handle and log all errors

        debug!("{}: Request succeeded", LOG_NAME);
    }
}
