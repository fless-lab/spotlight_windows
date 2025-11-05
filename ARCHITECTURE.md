# 🏗️ Architecture de Spotlight Windows

Ce document explique l'architecture technique de Spotlight Windows et les choix de conception.

## 📐 Vue d'ensemble

Spotlight Windows utilise une **architecture modulaire en couches** pour séparer les responsabilités et optimiser les performances.

```
┌─────────────────────────────────────────────────────────────────┐
│                        USER INTERFACE                            │
│                         (egui/eframe)                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  • Input handling (keyboard, mouse)                       │  │
│  │  • Result rendering (with icons, metadata)                │  │
│  │  • Theme management (dark mode)                           │  │
│  │  • 60 FPS rendering loop                                  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────────┐
│                      SEARCH ENGINE LAYER                         │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Fuzzy Matcher         Cache (Moka)      Scorer           │  │
│  │  ─────────────         ─────────────     ────────         │  │
│  │  • SkimMatcherV2       • LRU eviction    • Relevance      │  │
│  │  • Typo tolerance      • 1000 entries    • Recency        │  │
│  │  • Scoring             • TTL: 5 min      • Type bonus     │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────────┐
│                      INDEXING LAYER                              │
│  ┌────────────────┬────────────────┬───────────────────────┐    │
│  │   Scanner      │    Watcher     │   Index Manager       │    │
│  │   ────────     │   ─────────    │   ──────────────      │    │
│  │ • WalkDir      │ • notify crate │ • Tantivy engine      │    │
│  │ • Rayon para   │ • Real-time    │ • Schema: 7 fields    │    │
│  │ • Content ext  │ • Async queue  │ • BM25 ranking        │    │
│  │ • Ignore rules │ • Incremental  │ • 50MB heap writer    │    │
│  └────────────────┴────────────────┴───────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────────┐
│                     FILE SYSTEM (Windows)                        │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  • NTFS / FAT32                                           │  │
│  │  • File system events (create, modify, delete)            │  │
│  │  • Metadata (size, dates, attributes)                     │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## 🎯 Composants Principaux

### 1. User Interface (UI Layer)

**Technologie:** egui + eframe

**Responsabilités:**
- Rendu de la fenêtre principale
- Gestion des événements clavier/souris
- Affichage des résultats de recherche
- Thème et styling

**Fichiers:**
- `src/ui/app.rs` - Application principale
- `src/ui/theme.rs` - Configuration du thème
- `src/ui/mod.rs` - Module UI

**Optimisations:**
- Immediate mode GUI (pas de diffing DOM)
- Rendu GPU avec glow
- Clipping des résultats hors viewport
- Clone minimal pour éviter les borrows

### 2. Search Engine Layer

**Technologie:** Tantivy + fuzzy-matcher + moka

**Architecture:**

```
Query Input → Cache Check → Tantivy Search → Fuzzy Scoring → Cache Store → Results
                  ↓                                                ↑
              Cache Hit                                      Cache Miss
                  ↓                                                ↑
              Return                                         Continue ──┘
```

**Processus de recherche:**

1. **Query normalization** - Lowercase, trim
2. **Cache lookup** - O(1) avec hash map
3. **Tantivy search** - BM25 ranking
   - Query sur 3 champs: name, path, content
   - Wildcards automatiques (*query*)
   - Limite: 2x max_results
4. **Fuzzy matching** - Re-ranking avec SkimMatcherV2
   - Score sur nom ET path
   - Bonus récence (< 7 jours)
   - Bonus dossiers
5. **Filtering** - min_fuzzy_score
6. **Sorting** - Score décroissant
7. **Truncation** - max_results
8. **Cache storage** - Pour requêtes futures

**Fichiers:**
- `src/search/engine.rs` - Moteur principal
- `src/search/cache.rs` - Cache LRU
- `src/search/mod.rs` - Types de données

### 3. Indexing Layer

#### A. Scanner (Initial Indexing)

**Technologie:** ignore + rayon + walkdir

**Algorithme:**

```rust
for root_path in include_paths {
    WalkBuilder::new(root_path)
        .threads(num_cpus)
        .parallel_walk(|entry| {
            if should_exclude(entry) { skip }

            metadata = extract_metadata(entry)
            content = extract_content(entry) // Si texte

            entries.push(FileEntry {
                path, name, extension,
                size, modified,
                is_directory,
                content  // NOUVEAU
            })
        })

    // Indexation parallèle avec Rayon
    entries.par_iter().for_each(|entry| {
        tokio::spawn(indexer.add_file(entry))
    })

    indexer.commit()
}
```

**Optimisations:**
- Parallélisme maximal (tous les cores)
- Respect .gitignore automatique
- Skip des dossiers exclus tôt
- Content extraction limitée (1MB, 10k chars)
- Batch commit (pas un commit par fichier)

#### B. Watcher (Real-time Updates)

**Technologie:** notify (inotify sur Linux, ReadDirectoryChangesW sur Windows)

**Algorithme:**

```rust
watcher = RecommendedWatcher::new(handler)
for path in include_paths {
    watcher.watch(path, RecursiveMode::Recursive)
}

loop {
    event = rx.recv()
    match event {
        Create | Modify => {
            entry = extract_file_entry(event.path)
            indexer.add_file(entry)
            indexer.commit()
        }
        Remove => {
            indexer.remove_file(event.path)
            indexer.commit()
        }
    }
}
```

**Optimisations:**
- Debouncing automatique par notify
- Commit immédiat (< 100ms de latence)
- File content extraction async
- Skip des excluded paths

#### C. Index Manager (Tantivy)

**Schéma:**

```rust
Schema {
    path:         TEXT | STORED,      // Chemin complet
    name:         TEXT | STORED,      // Nom fichier
    extension:    STRING | STORED,    // Extension
    size:         U64 | INDEXED | STORED,
    modified:     DATE | INDEXED | STORED,
    is_directory: BOOL | INDEXED | STORED,
    content:      TEXT                // Contenu (NEW!)
}
```

**Configuration:**
- Writer heap: 50 MB
- Merge policy: Default (log structured)
- Reload policy: OnCommitWithDelay

**Fichiers:**
- `src/indexer/mod.rs` - Gestionnaire Tantivy
- `src/indexer/scanner.rs` - Scanner initial
- `src/indexer/watcher.rs` - File watcher

## 🔄 Flux de Données

### Indexation Initiale

```
1. Lancement app
   ↓
2. Load config
   ↓
3. Create Tantivy index
   ↓
4. Spawn scanner task
   ↓
5. Walk file system (parallel)
   ↓
6. Extract metadata + content
   ↓
7. Add to Tantivy (batch)
   ↓
8. Commit
   ↓
9. Log "✅ Scan terminé"
```

### Recherche en Temps Réel

```
1. User types "bug"
   ↓
2. UI calls search_engine.search("bug", 50)
   ↓
3. Check cache["bug"] → Miss
   ↓
4. Parse query: "bug*"
   ↓
5. Tantivy search on [name, path, content]
   ↓
6. Returns 100 results (2x limit)
   ↓
7. Fuzzy re-rank with SkimMatcherV2
   ↓
8. Filter (score >= 50)
   ↓
9. Sort by score DESC
   ↓
10. Truncate to 50
    ↓
11. Cache["bug"] = results
    ↓
12. Return to UI
    ↓
13. UI renders (60 FPS)
```

### File Change Detection

```
1. User creates "todo.txt"
   ↓
2. Windows notifies notify crate
   ↓
3. Event: Create("C:\Users\...\todo.txt")
   ↓
4. Watcher handler triggered
   ↓
5. Extract metadata
   ↓
6. Read content (< 1MB, is .txt)
   ↓
7. Create FileEntry { content: "Buy milk..." }
   ↓
8. indexer.add_file(entry)
   ↓
9. indexer.commit()
   ↓
10. Index updated (<100ms)
    ↓
11. Search "milk" now finds "todo.txt"!
```

## 🚀 Optimisations Clés

### 1. Content Extraction (Smart & Safe)

**Critères pour indexer le contenu:**
- ✅ Est un fichier (pas dossier)
- ✅ Taille < 1 MB
- ✅ Extension dans TEXT_EXTENSIONS (30+ types)
- ✅ UTF-8 valide
- ✅ Limité à 10k caractères

**Extensions supportées:**
```
txt, md, rs, toml, json, xml, yaml, yml
js, ts, py, go, c, cpp, h, hpp
java, cs, rb, php, html, css, scss
sh, bash, ps1, bat, cmd, log, ini, cfg
```

**Pourquoi ces limites ?**
- 1 MB: Évite de charger des logs géants en RAM
- 10k chars: Tantivy performant avec textes courts
- UTF-8 check: Pas de crash sur binaires

### 2. Parallel Everything

**Scanner:**
- `WalkBuilder.threads(num_cpus)` - Tous les cores
- `rayon::par_iter()` - Indexation parallèle
- `tokio::spawn()` - Async file add

**Search:**
- Tantivy est thread-safe par design
- Cache concurrent (Moka)
- UI async (pas de freeze)

### 3. Cache Strategy

**Moka Cache:**
- TTL: 5 minutes (auto-invalidation)
- Max: 1000 entrées (LRU eviction)
- Thread-safe (lock-free internally)
- Hit rate typique: 60-80%

**Pourquoi 5 minutes ?**
- Balance freshness vs performance
- Queries répétées (ex: "do", "doc", "docu", "docum")
- Invalide si index change (via commit)

### 4. Memory Management

**Index on Disk:**
- Tantivy stocke index sur disque
- RAM: seulement writer heap (50 MB)
- Searcher charge segments au besoin (mmap)

**UI:**
- Clone results pour éviter borrows
- Pas de retain de gros vecs
- egui recycle buffers

## 🎯 Architecture Future: Sentinel/Worker

**Vision:** Séparer indexation metadata (MFT) de l'indexation contenu

### Sentinel (Metadata + Queries)

**Rôle:**
- Index MFT Windows (comme Everything)
- Requêtes instantanées (< 1ms)
- File watching

**Tech:**
- Windows API: DeviceIoControl + FSCTL_ENUM_USN_DATA
- In-memory index (nom, taille, date)
- B-tree pour range queries

### Worker (Content Indexing)

**Rôle:**
- Indexation contenu (lourde)
- Extraction PDF/DOCX (sandboxed)
- Low priority (idle CPU)

**Tech:**
- Process séparé (isolation)
- Job queue (Kafka-like ou channel)
- Timeout par fichier (10s)
- Crash-safe (un PDF corrompu ne kill pas tout)

### Communication

```
Sentinel ←→ Worker
    ↓         ↓
  Query     Index
  Result    Content
    ↓         ↓
  Merge   Priority: Low
```

**Fusion des résultats:**
1. Query → Sentinel (metadata) [0-1ms]
2. Query → Worker (content) [5-20ms]
3. Merge + sort par score
4. Return top N

## 📊 Performance Targets

| Métrique | Actuel | Objectif Sentinel/Worker |
|----------|--------|--------------------------|
| Query latency | 8ms | < 1ms (metadata only) |
| Index 500k files | 45s | 10s (MFT) + 2min (content bg) |
| RAM usage | 100MB | 50MB (sentinel) + 200MB (worker) |
| CPU idle | ~1% | < 0.1% (sentinel) |

## 🔒 Sécurité & Robustesse

**Content Extraction:**
- Timeout: 10s par fichier (à venir)
- Sandbox: Process isolé (à venir)
- Size limits: 1MB hard limit
- UTF-8 validation: Pas de crash sur binaires

**Index Integrity:**
- Tantivy ACID (write-ahead log)
- Crash-safe commits
- Lock files (single writer)

**Resource Limits:**
- Max heap: 50 MB writer
- Max cache: 1000 entries
- Max content: 10k chars/file

## 📚 Technologies Détaillées

### Tantivy

**Pourquoi Tantivy ?**
- ✅ 2x plus rapide que Lucene
- ✅ Rust natif (memory safe)
- ✅ ACID transactions
- ✅ BM25 ranking proven
- ✅ Streaming indexing

**Schéma design:**
- TEXT fields: Tokenisés, stemming
- STRING fields: Exact match only
- Stored: Retourné dans results
- Indexed: Searchable

### egui

**Pourquoi egui ?**
- ✅ Immediate mode (simple)
- ✅ Pure Rust (no C++ binding)
- ✅ 60 FPS garanti
- ✅ Small binary (<10 MB)
- ✅ GPU accelerated

**vs Alternatives:**
- Tauri: Plus lourd (Chromium)
- Iced: Plus complexe (Elm architecture)
- druid: Moins mature

### Moka

**Pourquoi Moka ?**
- ✅ Lock-free (très rapide)
- ✅ TTL + LRU hybrid
- ✅ Async-aware
- ✅ Tiny overhead

**Cache hit examples:**
```
"doc"    → miss (8ms)
"docu"   → miss (8ms)
"doc"    → HIT (<1ms)  ← Repeat query
"docum"  → miss (8ms)
"doc"    → HIT (<1ms)  ← Still cached
```

## 🛠️ Maintenance

**Index rebuild:**
```bash
# Supprimer l'index
rm -rf ~/.local/share/spotlight_windows/index/

# Relancer l'app → re-scan auto
cargo run --release
```

**Monitor performance:**
```bash
RUST_LOG=debug,tantivy=info cargo run --release
```

**Profiling:**
```bash
cargo install flamegraph
cargo flamegraph
```

---

**Questions ? Consultez:**
- README.md - Guide utilisateur
- BUILD_INSTRUCTIONS.md - Compilation
- GitHub Issues - Support
