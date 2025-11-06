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

    /// Retourne le nombre d'entrées dans le cache
    #[cfg(test)]
    pub async fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Test de l'insertion et récupération dans le cache
    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache = SearchCache::new(10);

        let results = vec![SearchResult {
            path: PathBuf::from("/test/file.txt"),
            name: "file.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
            modified: 0,
            is_directory: false,
            score: 80.0,
        }];

        cache.set("test query".to_string(), results.clone()).await;

        let cached = cache.get("test query").await;
        assert!(cached.is_some());
        let cached_results = cached.unwrap();
        assert_eq!(cached_results.len(), 1);
        assert_eq!(cached_results[0].name, "file.txt");
    }

    /// Test de l'absence de résultat pour une clé inexistante
    #[tokio::test]
    async fn test_cache_miss() {
        let cache = SearchCache::new(10);

        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
    }

    /// Test de l'invalidation du cache
    #[tokio::test]
    async fn test_cache_clear() {
        let cache = SearchCache::new(10);

        let results = vec![SearchResult {
            path: PathBuf::from("/test/file.txt"),
            name: "file.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
            modified: 0,
            is_directory: false,
            score: 80.0,
        }];

        cache.set("test query".to_string(), results).await;

        // Vérifier que l'entrée existe
        assert!(cache.get("test query").await.is_some());

        // Effacer le cache
        cache.clear().await;

        // Vérifier que l'entrée n'existe plus
        assert!(cache.get("test query").await.is_none());
    }

    /// Test du comportement LRU (éviction des anciennes entrées)
    #[tokio::test]
    async fn test_cache_eviction() {
        let cache = SearchCache::new(2); // Capacité limitée à 2 entrées

        let results1 = vec![SearchResult {
            path: PathBuf::from("/test/file1.txt"),
            name: "file1.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
            modified: 0,
            is_directory: false,
            score: 80.0,
        }];

        let results2 = vec![SearchResult {
            path: PathBuf::from("/test/file2.txt"),
            name: "file2.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 2048,
            modified: 0,
            is_directory: false,
            score: 70.0,
        }];

        let results3 = vec![SearchResult {
            path: PathBuf::from("/test/file3.txt"),
            name: "file3.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 3072,
            modified: 0,
            is_directory: false,
            score: 90.0,
        }];

        cache.set("query1".to_string(), results1).await;
        cache.set("query2".to_string(), results2).await;

        // Les deux premières requêtes devraient être dans le cache
        assert!(cache.get("query1").await.is_some());
        assert!(cache.get("query2").await.is_some());

        // Ajouter une troisième requête devrait évincer la première (LRU)
        cache.set("query3".to_string(), results3).await;

        // Attendre un peu pour que l'éviction se produise
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Le cache devrait contenir au maximum 2 entrées
        let count = cache.entry_count().await;
        assert!(count <= 2, "Cache should have at most 2 entries, found {}", count);
    }

    /// Test de plusieurs requêtes identiques (idempotence)
    #[tokio::test]
    async fn test_cache_idempotence() {
        let cache = SearchCache::new(10);

        let results = vec![SearchResult {
            path: PathBuf::from("/test/file.txt"),
            name: "file.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
            modified: 0,
            is_directory: false,
            score: 80.0,
        }];

        // Insérer la même clé plusieurs fois
        cache.set("test".to_string(), results.clone()).await;
        cache.set("test".to_string(), results.clone()).await;
        cache.set("test".to_string(), results.clone()).await;

        // Attendre un peu pour que les insertions soient traitées
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Vérifier que la clé existe et retourne les bonnes valeurs
        let cached = cache.get("test").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }
}
