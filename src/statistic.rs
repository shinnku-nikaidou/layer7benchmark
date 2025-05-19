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

impl StatusCounter {
    pub fn get_grpc(&self) -> crate::server::commands::RequestCommandResultItem {
        crate::server::commands::RequestCommandResultItem {
            code_2: self.status_2xx.load(std::sync::atomic::Ordering::Relaxed),
            code_3: self.status_3xx.load(std::sync::atomic::Ordering::Relaxed),
            code_4: self.status_4xx.load(std::sync::atomic::Ordering::Relaxed),
            code_5: self.status_5xx.load(std::sync::atomic::Ordering::Relaxed),
            failure: self.status_other.load(std::sync::atomic::Ordering::Relaxed),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}

pub static STATISTIC: OnceCell<Statistic> = OnceCell::const_new();
