use std::sync::atomic::AtomicU64;
use tokio::sync::OnceCell;

#[derive(Default)]
pub struct Statistic {
    pub request_counter: AtomicU64,
    pub status_counter: StatusCounter,
    pub network_traffics: AtomicU64,
}

#[derive(Default)]
pub struct StatusCounter {
    pub status_2xx: AtomicU64,
    pub status_3xx: AtomicU64,
    pub status_4xx: AtomicU64,
    pub status_5xx: AtomicU64,
    pub status_other: AtomicU64,
}

pub static STATISTIC: OnceCell<Statistic> = OnceCell::const_new();
