//! Release notifications only. Downloads open in the user's browser; no silent installer.
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const RELEASES: &str = "https://github.com/bandarko/radio-crash-desktop/releases";
const API: &str = "https://api.github.com/repos/bandarko/radio-crash-desktop/releases/latest";
const INTERVAL: u64 = 6 * 60 * 60;
const MIN_CHECK_INTERVAL: u64 = 60;

// Recheck each launch even if an older successful result is cached. ETags
// still avoid redundant payloads; rapid relaunches share a one-minute throttle.
fn startup_delay(checked_at: u64, now: u64) -> Duration {
    let age = if checked_at <= now {
        now - checked_at
    } else {
        MIN_CHECK_INTERVAL
    };
    Duration::from_secs(MIN_CHECK_INTERVAL.saturating_sub(age).max(5))
}
#[derive(Debug)]
struct RateLimited(u64);
impl std::fmt::Display for RateLimited {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GitHub requested a {} second cooldown", self.0)
    }
}
impl std::error::Error for RateLimited {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(pub u32, pub u32, pub u32);
impl Version {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let parts: Vec<_> = s.split('.').collect();
        if parts.len() != 3
            || parts
                .iter()
                .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        Some(Self(
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    }
    pub fn text(self) -> String {
        format!("{}.{}.{}", self.0, self.1, self.2)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub tag: String,
    pub assets: Vec<String>,
}
impl Release {
    pub fn page(&self) -> String {
        format!("{RELEASES}/tag/{}", self.tag)
    }
    pub fn download(&self) -> String {
        let platform = if cfg!(target_os = "macos") {
            "macOS-universal"
        } else {
            "Windows-x64"
        };
        let name = format!("Radio-Crash-Desktop-{}-{platform}.zip", self.version.text());
        if self.assets.contains(&name) {
            format!("{RELEASES}/download/{}/{name}", self.tag)
        } else {
            self.page()
        }
    }
}
pub fn parse_release(value: &serde_json::Value) -> anyhow::Result<Release> {
    anyhow::ensure!(
        value["draft"] == false && value["prerelease"] == false,
        "Not a stable published release"
    );
    let tag = value["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing release tag"))?;
    let version =
        Version::parse(tag).ok_or_else(|| anyhow::anyhow!("Invalid stable release version"))?;
    // Restrict tags to canonical numbers so cached data cannot inject a URL path.
    anyhow::ensure!(
        tag == version.text() || tag == format!("v{}", version.text()),
        "Noncanonical release tag"
    );
    let assets = value["assets"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v["name"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    Ok(Release {
        version,
        tag: tag.to_owned(),
        assets,
    })
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct Cache {
    etag: Option<String>,
    checked_at: u64,
    release: Option<Release>,
}
#[derive(Clone)]
pub enum State {
    Idle,
    Checking,
    Current,
    Available(Release),
    Failed(String),
}
fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn cache_path() -> PathBuf {
    crate::settings::Settings::path().with_file_name("updates.json")
}
fn fetch(endpoint: &str, previous: &Cache) -> anyhow::Result<Cache> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(4))
        .timeout(Duration::from_secs(8))
        .user_agent(concat!("RadioCrashDesktop/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let mut request = client
        .get(endpoint)
        .header("Accept", "application/vnd.github+json");
    if let Some(tag) = &previous.etag {
        request = request.header("If-None-Match", tag);
    }
    let response = request.send()?;
    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        || (response.status() == reqwest::StatusCode::FORBIDDEN
            && response
                .headers()
                .get("x-ratelimit-remaining")
                .is_some_and(|v| v == "0"))
    {
        let retry = response
            .headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let reset = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(|t| t.saturating_sub(unix_time()));
        return Err(RateLimited(retry.unwrap_or(0).max(reset.unwrap_or(0)).max(60)).into());
    }
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        anyhow::ensure!(previous.release.is_some(), "Missing cached release");
        return Ok(Cache {
            checked_at: unix_time(),
            ..previous.clone()
        });
    }
    let response = response.error_for_status()?;
    let etag = response
        .headers()
        .get("etag")
        .and_then(|h| h.to_str().ok())
        .map(str::to_owned);
    let mut bytes = Vec::new();
    response.take(256 * 1024 + 1).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= 256 * 1024, "Release response too large");
    let release = parse_release(&serde_json::from_slice(&bytes)?)?;
    Ok(Cache {
        etag,
        checked_at: unix_time(),
        release: Some(release),
    })
}
pub struct Checker {
    pub state: State,
    cache: Cache,
    current: Version,
    receiver: mpsc::Receiver<anyhow::Result<Cache>>,
    sender: mpsc::Sender<anyhow::Result<Cache>>,
    worker: Option<thread::JoinHandle<()>>,
    next_check: Instant,
    last_attempt: Option<Instant>,
    blocked_until: Option<Instant>,
    pub revision: u64,
}
impl Checker {
    pub fn new() -> Self {
        let mut cache: Cache = std::fs::read(cache_path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        if cache.release.as_ref().is_some_and(|r| {
            Version::parse(&r.tag) != Some(r.version)
                || (r.tag != r.version.text() && r.tag != format!("v{}", r.version.text()))
        }) {
            cache = Cache::default();
        }
        let wait = startup_delay(cache.checked_at, unix_time());
        let current = Version::parse(env!("CARGO_PKG_VERSION")).expect("semantic package version");
        let state = match &cache.release {
            Some(r) if r.version > current => State::Available(r.clone()),
            Some(_) => State::Current,
            None => State::Idle,
        };
        let (sender, receiver) = mpsc::channel();
        Self {
            state,
            cache,
            current,
            receiver,
            sender,
            worker: None,
            next_check: Instant::now() + wait,
            last_attempt: None,
            blocked_until: None,
            revision: 0,
        }
    }
    pub fn busy(&self) -> bool {
        self.worker.is_some()
            || self
                .blocked_until
                .is_some_and(|until| Instant::now() < until)
    }
    pub fn request(&mut self) {
        if self.busy()
            || self
                .last_attempt
                .is_some_and(|t| t.elapsed() < Duration::from_secs(MIN_CHECK_INTERVAL))
        {
            return;
        }
        self.last_attempt = Some(Instant::now());
        self.state = State::Checking;
        let sender = self.sender.clone();
        let cache = self.cache.clone();
        self.worker = Some(thread::spawn(move || {
            let _ = sender.send(fetch(API, &cache));
        }));
    }
    pub fn tick(&mut self, enabled: bool) {
        if let Ok(result) = self.receiver.try_recv() {
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
            self.revision += 1;
            match result {
                Ok(cache) => {
                    self.cache = cache;
                    self.state = match &self.cache.release {
                        Some(r) if r.version > self.current => State::Available(r.clone()),
                        _ => State::Current,
                    };
                    self.next_check = Instant::now() + Duration::from_secs(INTERVAL);
                    let path = cache_path();
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Ok(bytes) = serde_json::to_vec(&self.cache) {
                        let _ = std::fs::write(path, bytes);
                    }
                }
                Err(error) => {
                    log::warn!("update check: {error:#}");
                    self.state =
                        State::Failed("Provjera trenutačno nije dostupna. Pokušaj kasnije.".into());
                    if let Some(rate) = error.downcast_ref::<RateLimited>() {
                        self.blocked_until =
                            Instant::now().checked_add(Duration::from_secs(rate.0));
                        self.next_check = self
                            .blocked_until
                            .unwrap_or(Instant::now() + Duration::from_secs(86400));
                    } else {
                        self.next_check = Instant::now() + Duration::from_secs(3600);
                    }
                }
            }
        }
        if enabled && Instant::now() >= self.next_check {
            self.request();
        }
    }
}
impl Drop for Checker {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub fn check_now() -> anyhow::Result<String> {
    let cache = fetch(API, &Cache::default())?;
    let release = cache.release.expect("fetched release");
    Ok(format!(
        "current={} latest={} page={} download={}",
        env!("CARGO_PKG_VERSION"),
        release.version.text(),
        release.page(),
        release.download()
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_checks_cached_releases_without_waiting_six_hours() {
        assert_eq!(
            startup_delay(1000, 1000 + 2 * 60 * 60),
            Duration::from_secs(5)
        );
        assert_eq!(startup_delay(1000, 1020), Duration::from_secs(40));
        assert_eq!(startup_delay(1000, 1000), Duration::from_secs(60));
        assert_eq!(startup_delay(0, 1000000), Duration::from_secs(5));
        assert_eq!(startup_delay(2000, 1000), Duration::from_secs(5));
    }
    #[test]
    fn stable_versions_and_safe_downloads() {
        assert!(Version::parse("v1.10.0") > Version::parse("1.9.9"));
        for bad in ["1.2", "1.2.3-beta", "v1.2.3/../../evil", "1.2.3?x=y"] {
            assert!(Version::parse(bad).is_none());
        }
        let value = serde_json::json!({"tag_name":"v1.2.0","draft":false,"prerelease":false,"html_url":"https://evil.example/","assets":[{"name":"Radio-Crash-Desktop-1.2.0-macOS-universal.zip","browser_download_url":"https://evil.example/"}]});
        let r = parse_release(&value).unwrap();
        assert_eq!(r.page(), format!("{RELEASES}/tag/v1.2.0"));
        assert!(r.download().starts_with(RELEASES));
        assert!(
            parse_release(
                &serde_json::json!({"tag_name":"v1.2.0","draft":true,"prerelease":false})
            )
            .is_err()
        );
        assert!(
            parse_release(
                &serde_json::json!({"tag_name":"v1.2.0","draft":false,"prerelease":true})
            )
            .is_err()
        );
    }
    #[test]
    fn conditional_http_check_preserves_cached_release() {
        use std::io::Write;
        use std::net::TcpListener;
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", server.local_addr().unwrap());
        let worker = thread::spawn(move || {
            for i in 0..2 {
                let (mut stream, _) = server.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = Vec::new();
                let mut b = [0u8; 1];
                while !request.ends_with(b"\r\n\r\n") {
                    stream.read_exact(&mut b).unwrap();
                    request.push(b[0]);
                }
                if i == 0 {
                    let body =
                        r#"{"tag_name":"v1.2.0","draft":false,"prerelease":false,"assets":[]}"#;
                    write!(stream,"HTTP/1.1 200 OK\r\nETag: \"release-test\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
                } else {
                    assert!(
                        String::from_utf8(request)
                            .unwrap()
                            .to_lowercase()
                            .contains("if-none-match: \"release-test\"")
                    );
                    write!(
                        stream,
                        "HTTP/1.1 304 Not Modified\r\nConnection: close\r\n\r\n"
                    )
                    .unwrap();
                }
            }
        });
        let first = fetch(&url, &Cache::default()).unwrap();
        let second = fetch(&url, &first).unwrap();
        assert_eq!(second.release.unwrap().version, Version(1, 2, 0));
        worker.join().unwrap();
    }
}
