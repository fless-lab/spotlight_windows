# 📦 Spotlight Windows - Binaires Précompilés

## 🚀 Installation Rapide

**Pas besoin de compiler !** Téléchargez directement le binaire pour votre plateforme.

## 🪟 Pour Windows (x86_64)

### Téléchargement Direct

```powershell
# Télécharger
Invoke-WebRequest -Uri "https://github.com/fless-lab/spotlight_windows/raw/claude/rust-desktop-spotlight-search-011CUoqZj9PhKNLXWzyaLPJn/bin/spotlight_windows.exe" -OutFile "spotlight_windows.exe"

# Lancer
.\spotlight_windows.exe
```

**Ou depuis le dépôt :**
```powershell
git clone https://github.com/fless-lab/spotlight_windows.git
cd spotlight_windows\bin
.\spotlight_windows.exe
```

### Alternative: Compiler sur Windows (pour hotkey global)

```powershell
# 1. Installer Rust (https://rustup.rs/)
# Télécharger et exécuter rustup-init.exe

# 2. Cloner le projet
git clone https://github.com/fless-lab/spotlight_windows.git
cd spotlight_windows

# 3. Build release (5 minutes la première fois)
cargo build --release

# 4. Lancer
.\target\release\spotlight_windows.exe
```

### Méthode 2: Cross-compiler depuis Linux (Avancé)

```bash
# Installer le target Windows
rustup target add x86_64-pc-windows-gnu

# Cross-compiler (nécessite mingw)
sudo apt-get install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu

# Le .exe sera dans target/x86_64-pc-windows-gnu/release/
```

## 🐧 Pour Linux (Binaire Disponible)

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
