pub mod scanner;
pub mod watcher;

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter};
use tokio::sync::RwLock;
use tracing::info;

use crate::config::Config;

/// Structure représentant un fichier indexé
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: i64,
    pub is_directory: bool,
    pub content: Option<String>, // Contenu du fichier pour recherche full-text
}

/// Gestionnaire de l'index Tantivy
pub struct Indexer {
    index: Index,
    writer: Arc<RwLock<IndexWriter>>,
    schema: Schema,
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl Indexer {
    /// Crée un nouvel indexeur
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let mut schema_builder = Schema::builder();

        // Options de tokenization pour recherche de sous-chaînes
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("ngram3") // On va créer ce tokenizer
                    .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions)
            )
            .set_stored();

        let text_options_basic = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("default")
                    .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions)
            )
            .set_stored();

        // Définition du schéma Tantivy
        schema_builder.add_text_field("path", text_options.clone());
        schema_builder.add_text_field("name", text_options.clone());
        schema_builder.add_text_field("extension", STRING | STORED);
        schema_builder.add_u64_field("size", INDEXED | STORED);
        schema_builder.add_date_field("modified", INDEXED | STORED);
        schema_builder.add_bool_field("is_directory", INDEXED | STORED);

        // NOUVEAU: Champ pour le contenu des fichiers (recherche full-text)
        schema_builder.add_text_field("content", text_options_basic);

        let schema = schema_builder.build();

        // Créer l'index dans un dossier local
        let index_path = Self::index_path();
        std::fs::create_dir_all(&index_path)?;

        // IMPORTANT : D'abord essayer d'OUVRIR l'index existant
        // Si ça échoue, alors créer un nouveau
        let index = Index::open_in_dir(&index_path)
            .or_else(|_| {
                info!("Création d'un nouvel index...");
                Index::create_in_dir(&index_path, schema.clone())
            })?;

        info!("Index chargé depuis: {:?}", index_path);

        // Enregistrer le tokenizer NGram pour recherche de sous-chaînes
        use tantivy::tokenizer::*;

        index.tokenizers().register(
            "ngram3",
            TextAnalyzer::builder(NgramTokenizer::new(2, 4, false).unwrap())
                .filter(LowerCaser)
                .build(),
        );

        // Writer avec 100MB de heap (augmenté pour PDF)
        let writer = index.writer(100_000_000)?;

        info!("Index créé avec succès: {:?}", index_path);

        Ok(Self {
            index,
            writer: Arc::new(RwLock::new(writer)),
            schema,
            config,
        })
    }

    /// Retourne le chemin de l'index
    fn index_path() -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("spotlight_windows");
        path.push("index");
        path
    }

    /// Ajoute un fichier à l'index
    pub async fn add_file(&self, entry: FileEntry) -> Result<()> {
        let path_field = self.schema.get_field("path").unwrap();
        let name_field = self.schema.get_field("name").unwrap();
        let extension_field = self.schema.get_field("extension").unwrap();
        let size_field = self.schema.get_field("size").unwrap();
        let modified_field = self.schema.get_field("modified").unwrap();
        let is_directory_field = self.schema.get_field("is_directory").unwrap();
        let content_field = self.schema.get_field("content").unwrap();

        // Créer le document avec la macro doc!
        let extension_str = entry.extension.as_deref().unwrap_or("");
        let content_str = entry.content.as_deref().unwrap_or("");

        let document = if entry.extension.is_some() {
            doc!(
                path_field => entry.path.to_string_lossy().as_ref(),
                name_field => entry.name.as_str(),
                extension_field => extension_str,
                size_field => entry.size,
                modified_field => tantivy::DateTime::from_timestamp_secs(entry.modified),
                is_directory_field => entry.is_directory,
                content_field => content_str
            )
        } else {
            doc!(
                path_field => entry.path.to_string_lossy().as_ref(),
                name_field => entry.name.as_str(),
                size_field => entry.size,
                modified_field => tantivy::DateTime::from_timestamp_secs(entry.modified),
                is_directory_field => entry.is_directory,
                content_field => content_str
            )
        };

        let writer = self.writer.write().await;
        writer.add_document(document)?;

        Ok(())
    }

    /// Commit les changements
    pub async fn commit(&self) -> Result<()> {
        let mut writer = self.writer.write().await;
        writer.commit()?;
        info!("Index committé avec succès");
        Ok(())
    }

    /// Supprime un fichier de l'index
    pub async fn remove_file(&self, path: &Path) -> Result<()> {
        let path_field = self.schema.get_field("path").unwrap();
        let term = Term::from_field_text(path_field, &path.to_string_lossy());

        let writer = self.writer.write().await;
        writer.delete_term(term);

        Ok(())
    }

    /// Retourne l'index pour les recherches
    pub fn get_index(&self) -> &Index {
        &self.index
    }

    /// Retourne le schéma
    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }

    /// Retourne le nombre de documents dans l'index
    pub fn num_documents(&self) -> u64 {
        if let Ok(reader) = self.index.reader() {
            let searcher = reader.searcher();
            searcher.num_docs()
        } else {
            0
        }
    }
}
