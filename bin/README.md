# 📦 Spotlight Windows - Binaire Précompilé

## 🚀 Installation Rapide

**Pas besoin de compiler !** Téléchargez directement l'exécutable :

### Linux (x86_64)

```bash
# Télécharger
wget https://github.com/fless-lab/spotlight_windows/raw/claude/rust-desktop-spotlight-search-011CUoqZj9PhKNLXWzyaLPJn/bin/spotlight_windows

# Rendre exécutable
chmod +x spotlight_windows

# Lancer
./spotlight_windows
```

### Depuis le dépôt Git

```bash
git clone https://github.com/fless-lab/spotlight_windows.git
cd spotlight_windows/bin
chmod +x spotlight_windows
./spotlight_windows
```

## 📊 Informations

**Version:** 0.2.0-dev
**Build:** Release (optimisé LTO)
**Plateforme:** Linux x86_64
**Taille:** ~18 MB (stripped)
**Dépendances:** Aucune (statically linked)

## ⚙️ Configuration

Au premier lancement, la configuration sera créée dans :
```
~/.config/spotlight_windows/config.toml
```

Éditez ce fichier pour personnaliser :
- Dossiers à indexer
- Exclusions
- Nombre de threads
- Taille du cache

## 🎯 Utilisation

1. **Lancer l'application**
   ```bash
   ./spotlight_windows
   ```

2. **Attendre l'indexation** (visible dans la console)
   - Peut prendre quelques minutes selon le nombre de fichiers
   - Progress affiché en temps réel

3. **Rechercher !**
   - Par nom de fichier
   - **Par contenu** (30+ formats texte)
   - Fuzzy matching (tolère les fautes)

## 🔧 Compilation depuis les sources

Si vous préférez compiler vous-même :

```bash
# Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Cloner et build
git clone https://github.com/fless-lab/spotlight_windows.git
cd spotlight_windows
cargo build --release

# L'exécutable sera dans target/release/
```

## 📚 Documentation

- **README.md** - Guide utilisateur complet
- **BUILD_INSTRUCTIONS.md** - Instructions de compilation
- **ARCHITECTURE.md** - Design technique
- **SUMMARY.md** - Résumé du projet

## ❓ Support

- [GitHub Issues](https://github.com/fless-lab/spotlight_windows/issues)
- [GitHub Discussions](https://github.com/fless-lab/spotlight_windows/discussions)

---

**Note:** Cet exécutable est compilé pour Linux. Pour Windows, compilez depuis les sources.
