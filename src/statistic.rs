use std::sync::{atomic::AtomicU64, Arc};
use tokio::sync::OnceCell;

#[derive(Default, Clone)]
pub struct Statistic {
    pub request_counter: Arc<AtomicU64>,
    pub status_counter: StatusCounter,
}

#[derive(Default, Clone)]
pub struct StatusCounter {
    pub status_2xx: Arc<AtomicU64>,
    pub status_3xx: Arc<AtomicU64>,
    pub status_4xx: Arc<AtomicU64>,
    pub status_5xx: Arc<AtomicU64>,
    pub status_other: Arc<AtomicU64>,
}

pub static STATISTIC: OnceCell<Statistic> = OnceCell::const_new();
