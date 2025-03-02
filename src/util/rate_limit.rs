/**
 * Rate limiting utility to stop certain site actions from happening too fast.
 */

use chrono::prelude::{ DateTime, Utc };
use dashmap::DashMap;
use tokio::sync::OnceCell;

pub static RATE_LIMIT_COUNTERS: OnceCell<DashMap<String, RateLimitCounter>> = OnceCell::const_new();

#[derive(Clone, Debug)]
pub struct RateLimitCounter {
    pub hits: Vec<DateTime<Utc>>,
}

pub fn init_rate_limits() {
    RATE_LIMIT_COUNTERS
        .set(DashMap::new())
        .expect("Rate limits already initialized.");
}

pub fn rate_limit_exceeded(id: &str, hit_max: usize, time_period_seconds: i64) -> bool {
    let rate_limits = RATE_LIMIT_COUNTERS.get().expect("Rate limits not initialized.");
    let now = Utc::now();
    
    let mut rate_limit = rate_limits.entry(id.to_string()).or_insert(RateLimitCounter { hits: vec![] });

    rate_limit.hits.retain(|timestamp| {
        now.signed_duration_since(*timestamp).num_seconds() < time_period_seconds
    });

    if rate_limit.hits.len() < hit_max {
        rate_limit.hits.push(now);
        false
    } else {
        true
    }
}
