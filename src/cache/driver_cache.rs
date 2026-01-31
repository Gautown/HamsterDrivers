// src/cache/driver_cache.rs
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use serde::{Serialize, Deserialize};
use lru::LruCache;

#[derive(Serialize, Deserialize, Clone)]
pub struct CachedDriverInfo {
    pub info: DriverInfo,
    pub timestamp: SystemTime,
    pub signature_cache: Option<SignatureCache>,
    pub file_hash: String,
}

pub struct DriverCache {
    cache: LruCache<String, CachedDriverInfo>,
    ttl: Duration,
    cache_file: PathBuf,
}

impl DriverCache {
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: LruCache::new(capacity),
            ttl: Duration::from_secs(ttl_seconds),
            cache_file: PathBuf::from("driver_cache.bin"),
        }
    }
    
    pub fn get(&mut self, key: &str) -> Option<&CachedDriverInfo> {
        if let Some(cached) = self.cache.get(key) {
            if cached.timestamp.elapsed().unwrap_or(Duration::MAX) < self.ttl {
                return Some(cached);
            }
        }
        None
    }
    
    pub fn store(&mut self, key: String, info: DriverInfo) {
        let cached = CachedDriverInfo {
            info,
            timestamp: SystemTime::now(),
            signature_cache: None,
            file_hash: String::new(),
        };
        self.cache.put(key, cached);
    }
    
    pub fn save_to_disk(&self) -> Result<(), String> {
        let data = bincode::serialize(&self.cache)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;
        
        fs::write(&self.cache_file, data)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;
        
        Ok(())
    }
    
    pub fn load_from_disk(&mut self) -> Result<(), String> {
        if self.cache_file.exists() {
            let data = fs::read(&self.cache_file)
                .map_err(|e| format!("Failed to read cache file: {}", e))?;
            
            let cache: LruCache<String, CachedDriverInfo> = bincode::deserialize(&data)
                .map_err(|e| format!("Failed to deserialize cache: {}", e))?;
            
            self.cache = cache;
        }
        Ok(())
    }
}