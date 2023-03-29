//! rs-vigil-reporter Vigil Reporter for Rust.

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::cmp::max;
use std::convert::TryFrom;
use std::io;
use std::thread;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as base64_encoder;
use base64::Engine;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use serde_json;
use sys_info::{cpu_num, loadavg, mem_info};

static LOG_NAME: &'static str = "Vigil Reporter";

pub const HTTP_CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

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
    report_uri: String,
    replica_id: String,
    interval: Duration,
    useragent: String,
    authorization: String,
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

        // Build thread manager context?
        match (self.probe_id, self.node_id, self.replica_id) {
            (Some(probe_id), Some(node_id), Some(replica_id)) => {
                let manager = ReporterManager {
                    report_uri: format!("{}/reporter/{}/{}/", self.url, probe_id, node_id),
                    replica_id: replica_id.to_owned(),
                    interval: self.interval,

                    useragent: format!(
                        "rs-{}/{}",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION")
                    ),

                    authorization: format!(
                        "Basic {}",
                        base64_encoder.encode(&format!(":{}", self.token))
                    ),
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
            LOG_NAME, &self.report_uri, payload
        );

        // Encode payload to string
        let payload_json = serde_json::to_vec(&payload).or(Err(()))?;

        // Generate request URI
        let request_uri = Uri::try_from(self.report_uri.as_str()).or(Err(()))?;

        // Acquire report response
        let mut response_sink = io::sink();

        let response = Request::new(&request_uri)
            .connect_timeout(Some(HTTP_CLIENT_TIMEOUT))
            .read_timeout(Some(HTTP_CLIENT_TIMEOUT))
            .write_timeout(Some(HTTP_CLIENT_TIMEOUT))
            .method(Method::POST)
            .header("User-Agent", &self.useragent)
            .header("Authorization", &self.authorization)
            .header("Content-Type", "application/json")
            .header("Content-Length", &payload_json.len())
            .body(&payload_json)
            .send(&mut response_sink);

        match response {
            Ok(response) => {
                let status_code = response.status_code();

                if status_code.is_success() {
                    debug!("{}: Request succeeded", LOG_NAME);

                    // Return with success
                    return Ok(());
                } else {
                    warn!("{}: Got non-OK status code: {}", LOG_NAME, status_code);
                }
            }
            Err(err) => error!("{}: Failed dispatching request: {}", LOG_NAME, err),
        }

        // Return with error
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
