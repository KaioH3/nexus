//! Rate limiting for Nexus Protocol.
//!
//! Implements token bucket algorithm for per-client rate limiting.

use std::time::Instant;

/// Rate limit configuration per client tier.
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub max_concurrent_executions: u32,
    pub compute_quota_bytes: u64,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            max_concurrent_executions: 4,
            compute_quota_bytes: 1_000_000_000, // 1GB
        }
    }
}

/// Token bucket state for a client.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    pub tokens: f64,
    pub last_refill: Instant,
    pub capacity: f64,
    pub refill_rate: f64, // tokens per second
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_second: f64) -> Self {
        Self {
            tokens: capacity as f64,
            last_refill: Instant::now(),
            capacity: capacity as f64,
            refill_rate: refill_per_second,
        }
    }

    /// Try to consume one token. Returns true if allowed.
    pub fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;

        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_refill = now;
    }

    /// Remaining tokens.
    pub fn remaining(&self) -> f64 {
        self.tokens
    }
}

/// Client rate limit tracker.
#[derive(Debug)]
pub struct RateLimitTracker {
    minute_bucket: TokenBucket,
    hour_bucket: TokenBucket,
    concurrent_executions: u32,
    max_concurrent: u32,
}

impl RateLimitTracker {
    pub fn new(rate_limit: &RateLimit) -> Self {
        Self {
            minute_bucket: TokenBucket::new(
                rate_limit.requests_per_minute,
                rate_limit.requests_per_minute as f64 / 60.0,
            ),
            hour_bucket: TokenBucket::new(
                rate_limit.requests_per_hour,
                rate_limit.requests_per_hour as f64 / 3600.0,
            ),
            concurrent_executions: 0,
            max_concurrent: rate_limit.max_concurrent_executions,
        }
    }

    /// Check if request is allowed. Returns true if allowed.
    pub fn check(&mut self) -> bool {
        self.minute_bucket.try_consume() && self.hour_bucket.try_consume()
    }

    /// Increment concurrent executions.
    pub fn start_execution(&mut self) -> bool {
        if self.concurrent_executions < self.max_concurrent {
            self.concurrent_executions += 1;
            true
        } else {
            false
        }
    }

    /// Decrement concurrent executions.
    pub fn end_execution(&mut self) {
        if self.concurrent_executions > 0 {
            self.concurrent_executions -= 1;
        }
    }

    /// Remaining requests in current window.
    pub fn remaining(&self) -> f64 {
        self.minute_bucket.remaining().min(self.hour_bucket.remaining())
    }
}

/// Error returned when rate limit is exceeded.
#[derive(Debug, Clone)]
pub struct RateLimitError {
    pub retry_after_secs: u32,
    pub limit_type: RateLimitType,
}

#[derive(Debug, Clone)]
pub enum RateLimitType {
    Minute,
    Hour,
    Concurrent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(10, 1.0);
        assert!(bucket.try_consume());
        assert_eq!(bucket.remaining(), 9.0);
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let mut tracker = RateLimitTracker::new(&RateLimit {
            requests_per_minute: 2,
            ..Default::default()
        });

        assert!(tracker.check());
        assert!(tracker.check());
        assert!(!tracker.check()); // Should be limited
    }

    #[test]
    fn test_concurrent_execution_limit() {
        let mut tracker = RateLimitTracker::new(&RateLimit {
            max_concurrent_executions: 2,
            ..Default::default()
        });

        assert!(tracker.start_execution());
        assert!(tracker.start_execution());
        assert!(!tracker.start_execution()); // Limit reached
        tracker.end_execution();
        assert!(tracker.start_execution()); // Now allowed
    }
}