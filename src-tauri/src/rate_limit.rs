use crate::job_control::{JobController, CANCELLED_BY_USER};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

pub const MIN_SPACING: Duration = Duration::from_secs(5);
pub const BASE_COOLDOWN: Duration = Duration::from_secs(60);
pub const MAX_COOLDOWN: Duration = Duration::from_secs(900);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitKind {
    Hard,
    Soft,
}

pub struct RateLimiter {
    next_allowed: Mutex<Option<Instant>>,
    consecutive_limits: AtomicU32,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            next_allowed: Mutex::new(None),
            consecutive_limits: AtomicU32::new(0),
        }
    }
}

pub fn classify(err: &str) -> Option<RateLimitKind> {
    let lower = err.to_ascii_lowercase();
    if lower.contains("429") || lower.contains("too many requests") {
        Some(RateLimitKind::Hard)
    } else if lower.contains("403") || lower.contains("forbidden") {
        Some(RateLimitKind::Soft)
    } else {
        None
    }
}

fn lock_next_allowed(
    slot: &Mutex<Option<Instant>>,
) -> std::sync::MutexGuard<'_, Option<Instant>> {
    slot.lock().unwrap_or_else(|e| e.into_inner())
}

impl RateLimiter {
    pub fn remaining(&self, now: Instant) -> Duration {
        match *lock_next_allowed(&self.next_allowed) {
            Some(until) if until > now => until - now,
            _ => Duration::ZERO,
        }
    }

    pub fn record_rate_limit(&self, now: Instant) {
        let n = self.consecutive_limits.fetch_add(1, Ordering::SeqCst) + 1;
        let shift = n.saturating_sub(1).min(8);
        let cooldown_secs = BASE_COOLDOWN
            .as_secs()
            .saturating_mul(1u64 << shift)
            .min(MAX_COOLDOWN.as_secs());
        *lock_next_allowed(&self.next_allowed) = Some(now + Duration::from_secs(cooldown_secs));
    }

    pub fn record_success(&self, now: Instant) {
        self.consecutive_limits.store(0, Ordering::SeqCst);
        *lock_next_allowed(&self.next_allowed) = Some(now + MIN_SPACING);
    }

    pub fn friendly_message(&self, now: Instant) -> String {
        friendly_rate_limit_message(self.remaining(now))
    }
}

pub fn friendly_rate_limit_message(remaining: Duration) -> String {
    let mins = remaining.as_secs().div_ceil(60).max(1);
    format!("YouTube is rate limiting this machine. Try again in {mins}m.")
}

pub fn cooldown_message(remaining: Duration) -> String {
    let secs = remaining.as_secs().max(1);
    format!("Waiting {secs}s — YouTube cooldown")
}

pub fn wait_until_allowed(app: &AppHandle) -> Result<(), String> {
    let limiter = app.state::<RateLimiter>();
    let jobs = app.state::<JobController>();
    let mut last_emitted_secs: Option<u64> = None;

    loop {
        if jobs.is_cancelled() {
            return Err(CANCELLED_BY_USER.into());
        }

        let remaining = limiter.remaining(Instant::now());
        if remaining.is_zero() {
            return Ok(());
        }

        let secs = remaining.as_secs().max(1);
        if last_emitted_secs != Some(secs) {
            crate::emit_progress_from(app, 2.0, &cooldown_message(remaining));
            last_emitted_secs = Some(secs);
        }

        std::thread::sleep(Duration::from_millis(250));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_detects_hard_429_and_too_many_requests() {
        assert_eq!(
            classify("HTTP Error 429: Too Many Requests"),
            Some(RateLimitKind::Hard)
        );
        assert_eq!(
            classify("ERROR: unable to download video data: HTTP Error 429"),
            Some(RateLimitKind::Hard)
        );
        assert_eq!(
            classify("too many requests from this IP"),
            Some(RateLimitKind::Hard)
        );
    }

    #[test]
    fn classify_detects_soft_403_forbidden() {
        assert_eq!(
            classify("Command failed: ERROR: unable to download video data: HTTP Error 403: Forbidden"),
            Some(RateLimitKind::Soft)
        );
        assert_eq!(classify("HTTP Error 403"), Some(RateLimitKind::Soft));
        assert_eq!(classify("Access forbidden"), Some(RateLimitKind::Soft));
    }

    #[test]
    fn classify_ignores_unrelated_and_cancel_errors() {
        assert_eq!(classify("Cancelled by user"), None);
        assert_eq!(classify("Downloaded file not found at C:\\tmp\\dl.mp4"), None);
        assert_eq!(classify("Invalid yt-dlp JSON: expected value"), None);
        assert_eq!(classify(""), None);
    }

    #[test]
    fn first_rate_limit_uses_base_cooldown() {
        let limiter = RateLimiter::default();
        let t0 = Instant::now();
        limiter.record_rate_limit(t0);
        assert_eq!(limiter.remaining(t0), BASE_COOLDOWN);
        assert_eq!(
            limiter.remaining(t0 + Duration::from_secs(20)),
            Duration::from_secs(40)
        );
        assert!(limiter.remaining(t0 + BASE_COOLDOWN).is_zero());
    }

    #[test]
    fn backoff_doubles_and_caps_at_max() {
        let limiter = RateLimiter::default();
        let t0 = Instant::now();

        limiter.record_rate_limit(t0);
        assert_eq!(limiter.remaining(t0), Duration::from_secs(60));

        limiter.record_rate_limit(t0);
        assert_eq!(limiter.remaining(t0), Duration::from_secs(120));

        limiter.record_rate_limit(t0);
        assert_eq!(limiter.remaining(t0), Duration::from_secs(240));

        limiter.record_rate_limit(t0);
        assert_eq!(limiter.remaining(t0), Duration::from_secs(480));

        limiter.record_rate_limit(t0);
        assert_eq!(limiter.remaining(t0), MAX_COOLDOWN);

        limiter.record_rate_limit(t0);
        assert_eq!(limiter.remaining(t0), MAX_COOLDOWN);
    }

    #[test]
    fn success_resets_streak_and_applies_min_spacing() {
        let limiter = RateLimiter::default();
        let t0 = Instant::now();

        limiter.record_rate_limit(t0);
        limiter.record_rate_limit(t0);
        limiter.record_success(t0);
        assert_eq!(limiter.remaining(t0), MIN_SPACING);
        assert!(limiter.remaining(t0 + MIN_SPACING).is_zero());

        limiter.record_rate_limit(t0);
        assert_eq!(limiter.remaining(t0), BASE_COOLDOWN);
    }

    #[test]
    fn unused_limiter_has_no_wait() {
        let limiter = RateLimiter::default();
        assert!(limiter.remaining(Instant::now()).is_zero());
    }

    #[test]
    fn friendly_message_rounds_up_to_whole_minutes() {
        assert_eq!(
            friendly_rate_limit_message(Duration::from_secs(1)),
            "YouTube is rate limiting this machine. Try again in 1m."
        );
        assert_eq!(
            friendly_rate_limit_message(Duration::from_secs(60)),
            "YouTube is rate limiting this machine. Try again in 1m."
        );
        assert_eq!(
            friendly_rate_limit_message(Duration::from_secs(61)),
            "YouTube is rate limiting this machine. Try again in 2m."
        );
        assert_eq!(
            friendly_rate_limit_message(MAX_COOLDOWN),
            "YouTube is rate limiting this machine. Try again in 15m."
        );
    }

    #[test]
    fn cooldown_message_shows_remaining_seconds() {
        assert_eq!(
            cooldown_message(Duration::from_secs(47)),
            "Waiting 47s — YouTube cooldown"
        );
        assert_eq!(
            cooldown_message(Duration::from_millis(200)),
            "Waiting 1s — YouTube cooldown"
        );
    }
}
