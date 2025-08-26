use crate::parser_config::ParserConfig;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Cache type for storing ParserConfig instances (wrapped in Arc)
pub type ConfigCache = Arc<RwLock<HashMap<String, Arc<ParserConfig>>>>;

// Caching interface for ParserConfig
#[derive(Clone)]
pub struct ParserConfigCache {
    cache: ConfigCache,
}

impl ParserConfigCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a ParserConfig by program name.
    /// If not in cache, load from filesystem and cache it.
    pub fn get_config(&self, program: &str) -> Result<Arc<ParserConfig>, String> {
        // First, try to read from cache
        {
            let cache = self.cache.read().unwrap();
            if let Some(config) = cache.get(program) {
                tracing::debug!(program = %program, "Config loaded from cache");
                return Ok(config.clone());
            }
        }

        // Not in cache, load from filesystem
        tracing::debug!(program = %program, "Config not in cache, loading from filesystem");
        let config_path = format!("configs/{}.toml", program);
        let config = ParserConfig::from_toml_file(&config_path)?;

        let arc = Arc::new(config);

        // Store in cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(program.to_string(), arc.clone());
            tracing::debug!(program = %program, cache_size = cache.len(), "Config cached");
        }

        Ok(arc)
    }

    /// Clear the cache (useful for testing or cache invalidation)
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Get cache size (useful for monitoring)
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }
}
