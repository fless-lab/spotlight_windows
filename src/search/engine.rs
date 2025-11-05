use super::cache::SearchCache;
use super::SearchResult;
use crate::config::Config;
use crate::indexer::Indexer;
use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Value;
use tantivy::{ReloadPolicy, TantivyDocument};
use tracing::debug;

/// Moteur de recherche ultra-rapide
pub struct SearchEngine {
    indexer: Arc<Indexer>,
    cache: SearchCache,
    fuzzy_matcher: SkimMatcherV2,
    config: Arc<Config>,
}

impl SearchEngine {
    pub fn new(indexer: Arc<Indexer>, config: Arc<Config>) -> Self {
        let cache = SearchCache::new(config.search.cache_size);

        Self {
            indexer,
            cache,
            fuzzy_matcher: SkimMatcherV2::default(),
            config,
        }
    }

    /// Recherche des fichiers
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        if query.is_empty() {
            return Ok(vec![]);
        }

        // Vérifier le cache
        if let Some(cached) = self.cache.get(query).await {
            debug!("Cache hit pour: {}", query);
            return Ok(cached);
        }

        let start = std::time::Instant::now();

        // Recherche dans l'index Tantivy
        let results = self.search_tantivy(query, max_results).await?;

        // Appliquer le fuzzy matching pour améliorer le ranking
        let mut scored_results = self.apply_fuzzy_scoring(query, results);

        // Trier par score décroissant
        scored_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Limiter les résultats
        scored_results.truncate(max_results);

        debug!(
            "Recherche terminée en {:.2}ms: {} résultats",
            start.elapsed().as_millis(),
            scored_results.len()
        );

        // Mettre en cache
        self.cache.set(query.to_string(), scored_results.clone()).await;

        Ok(scored_results)
    }

    /// Recherche dans l'index Tantivy
    async fn search_tantivy(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let index = self.indexer.get_index();
        let schema = self.indexer.get_schema();

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let searcher = reader.searcher();

        // Créer une query sur les champs name et path
        let name_field = schema.get_field("name").unwrap();
        let path_field = schema.get_field("path").unwrap();

        let query_parser = QueryParser::for_index(index, vec![name_field, path_field]);

        // Parser la query (avec wildcards automatiques)
        let query_str = format!("{}*", query);
        let tantivy_query = query_parser.parse_query(&query_str)?;

        // Rechercher
        let top_docs = searcher.search(&tantivy_query, &TopDocs::with_limit(limit * 2))?;

        let mut results = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let path = retrieved_doc
                .get_first(path_field)
                .and_then(|v| v.as_str())
                .map(PathBuf::from)
                .unwrap_or_default();

            let name = retrieved_doc
                .get_first(name_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let extension_field = schema.get_field("extension").unwrap();
            let extension = retrieved_doc
                .get_first(extension_field)
                .and_then(|v| v.as_str())
                .map(String::from);

            let size_field = schema.get_field("size").unwrap();
            let size = retrieved_doc
                .get_first(size_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let modified_field = schema.get_field("modified").unwrap();
            let modified = retrieved_doc
                .get_first(modified_field)
                .and_then(|v| v.as_datetime())
                .map(|d| d.into_timestamp_secs())
                .unwrap_or(0);

            let is_directory_field = schema.get_field("is_directory").unwrap();
            let is_directory = retrieved_doc
                .get_first(is_directory_field)
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            results.push(SearchResult {
                path,
                name,
                extension,
                size,
                modified,
                is_directory,
                score: 0.0, // Sera calculé par fuzzy matching
            });
        }

        Ok(results)
    }

    /// Applique le fuzzy matching pour améliorer le scoring
    fn apply_fuzzy_scoring(&self, query: &str, mut results: Vec<SearchResult>) -> Vec<SearchResult> {
        for result in &mut results {
            // Calculer le score fuzzy sur le nom
            let name_score = self
                .fuzzy_matcher
                .fuzzy_match(&result.name.to_lowercase(), &query.to_lowercase())
                .unwrap_or(0);

            // Calculer le score fuzzy sur le path
            let path_score = self
                .fuzzy_matcher
                .fuzzy_match(&result.path.to_string_lossy().to_lowercase(), &query.to_lowercase())
                .unwrap_or(0);

            // Le meilleur score l'emporte
            let fuzzy_score = name_score.max(path_score);

            // Bonus pour les dossiers
            let directory_bonus = if result.is_directory { 10 } else { 0 };

            // Bonus pour les fichiers récents
            let now = chrono::Utc::now().timestamp();
            let age_days = (now - result.modified) / 86400;
            let recency_bonus = if age_days < 7 {
                20
            } else if age_days < 30 {
                10
            } else {
                0
            };

            result.score = (fuzzy_score + directory_bonus + recency_bonus) as f32;
        }

        // Filtrer les résultats avec un score trop faible
        results.retain(|r| r.score >= self.config.search.min_fuzzy_score as f32);

        results
    }
}
