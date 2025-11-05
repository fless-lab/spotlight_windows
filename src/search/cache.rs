use super::SearchResult;
use moka::future::Cache;
use std::sync::Arc;

/// Cache LRU ultra-rapide pour les résultats de recherche
pub struct SearchCache {
    cache: Arc<Cache<String, Vec<SearchResult>>>,
}

impl SearchCache {
    /// Crée un nouveau cache
    pub fn new(max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(std::time::Duration::from_secs(300)) // 5 minutes
            .build();

        Self {
            cache: Arc::new(cache),
        }
    }

    /// Récupère des résultats du cache
    pub async fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        self.cache.get(&query.to_string()).await
    }

    /// Met en cache des résultats
    pub async fn set(&self, query: String, results: Vec<SearchResult>) {
        self.cache.insert(query, results).await;
    }

    /// Efface le cache
    #[allow(dead_code)]
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }
}
