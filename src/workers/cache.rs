use mlua::prelude::*;

/// The options for the cache.
#[derive(Debug, Clone)]
pub struct CacheOptions {
    /// The maximum lifetime of a cached value in seconds.
    /// By default, this is 86400 seconds (24 hours).
    /// When the lifetime of a cached value is exceeded without been accessed, it will be removed.
    pub max_lifetime: u64,
    /// The maximum number of entries in the cache.
    /// By default, this is 1000 entries.
    /// If the cache is full, the least recently used entry will be removed.
    pub max_size: u64,
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self {
            max_lifetime: 86400,
            max_size: 1000,
        }
    }
}