use std::{collections::HashMap, env, fs, path::Path, process, thread, time::Duration};

use serde_json::{json, Value};

const DATA_DIR: &str = "/data";
const IP_FILE: &str = "/data/last_ip.txt";
const TIMEOUT: Duration = Duration::from_secs(10);

/// One DNS record to keep up to date: `sub` is "@" for the root domain.
#[derive(Clone)]
struct Host {
    domain: String,
    sub: String,
}

impl Host {
    /// Parses `sub.example.com`, `example.com` (root) or `sub@example.co.uk`.
    /// Without an `@`, the registered domain is assumed to be the last two labels.
    fn parse(entry: &str) -> Result<Host, String> {
        if let Some((sub, domain)) = entry.split_once('@') {
            if domain.is_empty() || !domain.contains('.') {
                return Err(format!("invalid host entry: {entry}"));
            }
            let sub = if sub.is_empty() { "@" } else { sub };
            return Ok(Host { domain: domain.into(), sub: sub.into() });
        }
        let labels: Vec<&str> = entry.split('.').collect();
        if labels.len() < 2 || labels.iter().any(|l| l.is_empty()) {
            return Err(format!("invalid host entry: {entry}"));
        }
        let split = labels.len() - 2;
        let sub = if split == 0 { "@".to_string() } else { labels[..split].join(".") };
        Ok(Host { domain: labels[split..].join("."), sub })
    }

    /// Full hostname, used for logging and as the key in the state file.
    fn fqdn(&self) -> String {
        if self.sub == "@" {
            self.domain.clone()
        } else {
            format!("{}.{}", self.sub, self.domain)
        }
    }
}

struct Config {
    api_key: String,
    secret: String,
    hosts: Vec<Host>,
    interval: Duration,
    ip_url: String,
}

fn optional(name: &str) -> Option<String> {
    env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn fail(msg: &str) -> ! {
    eprintln!("{msg}");
    process::exit(1);
}

fn required(name: &str) -> String {
    optional(name).unwrap_or_else(|| fail(&format!("Missing required environment variable: {name}")))
}

impl Config {
    fn from_env() -> Self {
        let interval = match optional("CHECK_INTERVAL") {
            Some(v) => v
                .parse()
                .unwrap_or_else(|_| fail(&format!("CHECK_INTERVAL must be a number of seconds, got: {v}"))),
            None => 300,
        };

        let hosts = match optional("HOSTS") {
            Some(list) => list
                .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .map(|s| Host::parse(s).unwrap_or_else(|e| fail(&e)))
                .collect(),
            // Backwards compatible single-record configuration
            None => vec![Host { domain: required("DOMAIN"), sub: required("SUBDOMAIN") }],
        };
        if hosts.is_empty() {
            fail("HOSTS does not contain any entries");
        }

        Config {
            api_key: required("PORKBUN_API_KEY"),
            secret: required("PORKBUN_SECRET_API_KEY"),
            hosts,
            interval: Duration::from_secs(interval),
            ip_url: optional("IP_CHECK_URL").unwrap_or_else(|| "https://api.ipify.org".into()),
        }
    }
}

fn get_public_ip(cfg: &Config) -> Result<String, String> {
    let body = ureq::get(&cfg.ip_url)
        .timeout(TIMEOUT)
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;
    Ok(body.trim().to_string())
}

fn update_porkbun(cfg: &Config, host: &Host, ip: &str) -> Result<(), String> {
    // Porkbun expects the subdomain segment to be omitted for the root domain.
    let url = if host.sub == "@" {
        format!("https://api.porkbun.com/api/json/v3/dns/editByNameType/{}/A", host.domain)
    } else {
        format!(
            "https://api.porkbun.com/api/json/v3/dns/editByNameType/{}/A/{}",
            host.domain, host.sub
        )
    };
    let payload = json!({
        "apikey": cfg.api_key,
        "secretapikey": cfg.secret,
        "content": ip,
        "ttl": "300",
    });

    println!("Updating DNS: {} with IP {ip}", host.fqdn());

    let body = match ureq::post(&url).timeout(TIMEOUT).send_json(payload) {
        Ok(resp) => resp.into_string().map_err(|e| e.to_string())?,
        Err(ureq::Error::Status(code, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            return Err(format!("HTTP {code}: {text}"));
        }
        Err(e) => return Err(e.to_string()),
    };
    println!("API Response ({}): {body}", host.fqdn());

    let parsed: Value = serde_json::from_str(&body).map_err(|e| format!("invalid JSON: {e}"))?;
    match parsed.get("status").and_then(Value::as_str) {
        Some("SUCCESS") => Ok(()),
        _ => Err(format!("Porkbun reported failure: {body}")),
    }
}

/// State file format: one `<fqdn> <ip>` pair per line.
fn load_state() -> HashMap<String, String> {
    fs::read_to_string(IP_FILE)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once(' '))
        .map(|(h, ip)| (h.trim().to_string(), ip.trim().to_string()))
        .collect()
}

fn save_state(state: &HashMap<String, String>) -> Result<(), String> {
    if !Path::new(DATA_DIR).exists() {
        fs::create_dir_all(DATA_DIR).map_err(|e| e.to_string())?;
    }
    let mut lines: Vec<String> = state.iter().map(|(h, ip)| format!("{h} {ip}")).collect();
    lines.sort();
    fs::write(IP_FILE, lines.join("\n") + "\n").map_err(|e| e.to_string())
}

fn main() {
    let cfg = Config::from_env();
    let mut state = load_state();

    println!(
        "Watching {} record(s): {}",
        cfg.hosts.len(),
        cfg.hosts.iter().map(Host::fqdn).collect::<Vec<_>>().join(", ")
    );

    loop {
        match get_public_ip(&cfg) {
            Err(e) => eprintln!("Error: {e}"),
            Ok(ip) => {
                let mut changed = false;
                for host in &cfg.hosts {
                    let key = host.fqdn();
                    if state.get(&key).map(String::as_str) == Some(ip.as_str()) {
                        println!("No change: {key} {ip}");
                        continue;
                    }
                    // A failure for one record must not stop the others.
                    match update_porkbun(&cfg, host, &ip) {
                        Ok(()) => {
                            println!("IP updated: {key} {ip}");
                            state.insert(key, ip.clone());
                            changed = true;
                        }
                        Err(e) => eprintln!("Error updating {key}: {e}"),
                    }
                }
                if changed {
                    if let Err(e) = save_state(&state) {
                        eprintln!("Error saving state: {e}");
                    }
                }
            }
        }
        thread::sleep(cfg.interval);
    }
}
