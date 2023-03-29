//! rs-vigil-reporter Vigil Reporter for Rust.

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::cmp::max;
use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::redirect::Policy as RedirectPolicy;
use reqwest::StatusCode;
use sys_info::{cpu_num, loadavg, mem_info};

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
    report_url: String,
    auth_token: String,
    replica_id: String,
    interval: Duration,
    client: Client,
}

#[derive(Serialize, Debug)]
struct ReportPayload<'a> {
    replica: &'a str,
    interval: u64,
    load: ReportPayloadLoad,
}

#[derive(Serialize, Debug)]
struct ReportPayloadLoad {
    cpu: f32,
    ram: f32,
}

impl<'a> Reporter<'a> {
    pub fn new(url: &'a str, token: &'a str) -> ReporterBuilder<'a> {
        ReporterBuilder {
            reporter: Reporter {
                url: url,
                token: token,
                probe_id: None,
                node_id: None,
                replica_id: None,
                interval: Duration::from_secs(30),
            },
        }
    }

    pub fn run(&self) -> Result<(), ()> {
        debug!("{}: Will run using URL: {}", LOG_NAME, self.url);

        // Build HTTP client
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(RedirectPolicy::none())
            .gzip(true)
            .build();

        // Build thread manager context?
        match (self.probe_id, self.node_id, self.replica_id, http_client) {
            (Some(probe_id), Some(node_id), Some(replica_id), Ok(client)) => {
                let manager = ReporterManager {
                    report_url: format!("{}/reporter/{}/{}/", self.url, probe_id, node_id),
                    auth_token: self.token.to_owned(),
                    replica_id: replica_id.to_owned(),
                    interval: self.interval,
                    client: client,
                };

                // Spawn thread
                thread::Builder::new()
                    .name("vigil-reporter".to_string())
                    .spawn(move || manager.run())
                    .or(Err(()))
                    .and(Ok(()))
            }
            _ => Err(()),
        }
    }
}

impl<'a> ReporterBuilder<'a> {
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
            if self.report().is_err() == true {
                warn!(
                    "{}: Last report failed, trying again sooner than usual",
                    LOG_NAME
                );

                // Try reporting again after half the interval (this report failed)
                thread::sleep(self.interval / 2);

                self.report().ok();
            }

            thread::sleep(self.interval);
        }
    }

    fn report(&self) -> Result<(), ()> {
        debug!("{}: Will dispatch request", LOG_NAME);

        // Generate report payload
        let payload = ReportPayload {
            replica: &self.replica_id,
            interval: self.interval.as_secs(),
            load: ReportPayloadLoad {
                cpu: Self::get_load_cpu(),
                ram: Self::get_load_ram(),
            },
        };

        debug!(
            "{}: Will send request to URL: {} with payload: {:?}",
            LOG_NAME, &self.report_url, payload
        );

        // Submit report payload
        let response = self
            .client
            .post(&self.report_url)
            .basic_auth("", Some(&self.auth_token))
            .json(&payload)
            .send();

        match response {
            Ok(response_inner) => {
                let status = response_inner.status();

                if status == StatusCode::OK {
                    debug!("{}: Request succeeded", LOG_NAME);

                    return Ok(());
                } else {
                    warn!("{}: Got non-OK status code: {}", LOG_NAME, status);
                }
            }
            Err(err) => error!("{}: Failed dispatching request: {}", LOG_NAME, err),
        }

        Err(())
    }

    fn get_load_cpu() -> f32 {
        match (cpu_num(), loadavg()) {
            (Ok(cpu_num_value), Ok(loadavg_value)) => {
                (loadavg_value.one / (max(cpu_num_value, 1) as f64)) as f32
            }
            _ => 0.00,
        }
    }

    fn get_load_ram() -> f32 {
        if let Ok(mem_info_value) = mem_info() {
            1.00 - ((mem_info_value.avail as f32) / (mem_info_value.total as f32))
        } else {
            0.00
        }
    }
}
