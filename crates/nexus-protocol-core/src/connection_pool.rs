//! Connection pool for high-performance client reuse.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub max_idle_time: Duration,
    pub connection_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            max_idle_time: Duration::from_secs(300),
            connection_timeout: Duration::from_secs(5),
        }
    }
}

pub struct PooledConnection<T: Clone> {
    conn: T,
    pool: Arc<ConnectionPool<T>>,
    created_at: Instant,
}

impl<T: Clone> PooledConnection<T> {
    pub fn inner(&self) -> &T {
        &self.conn
    }

    pub fn into_inner(self) -> T {
        self.conn.clone()
    }

    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.created_at)
    }
}

impl<T: Clone> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if self.age() < self.pool.config.max_idle_time {
            if let Ok(mut guard) = self.pool.connections.lock() {
                if guard.len() < self.pool.config.max_connections {
                    guard.push_back(self.conn.clone());
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ConnectionPool<T: Clone> {
    connections: Mutex<VecDeque<T>>,
    config: PoolConfig,
}

impl<T: Clone> ConnectionPool<T> {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Mutex::new(VecDeque::new()),
            config: PoolConfig {
                max_connections,
                ..Default::default()
            },
        }
    }

    pub fn get(&self) -> Option<PooledConnection<T>> {
        if let Ok(mut guard) = self.connections.lock() {
            guard.pop_front().map(|conn| {
                PooledConnection {
                    conn,
                    pool: Arc::new(Self {
                        connections: Mutex::new(VecDeque::new()),
                        config: self.config.clone(),
                    }),
                    created_at: Instant::now(),
                }
            })
        } else {
            None
        }
    }

    pub fn return_connection(&self, conn: T) {
        if let Ok(mut guard) = self.connections.lock() {
            if guard.len() < self.config.max_connections {
                guard.push_back(conn);
            }
        }
    }

    pub fn idle_count(&self) -> usize {
        self.connections.lock().map(|g| g.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_basic() {
        let pool: ConnectionPool<String> = ConnectionPool::new(5);
        pool.return_connection("test".to_string());
        let conn = pool.get();
        assert!(conn.is_some());
    }

    #[test]
    fn test_pool_max() {
        let pool: ConnectionPool<String> = ConnectionPool::new(2);
        pool.return_connection("1".to_string());
        pool.return_connection("2".to_string());
        pool.return_connection("3".to_string());
        assert_eq!(pool.idle_count(), 2);
    }
}