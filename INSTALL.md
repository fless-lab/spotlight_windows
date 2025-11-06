# 📦 Installation de Spotlight Windows

Guide d'installation simple pour les utilisateurs.

---

## 🚀 Installation Rapide (Recommandé)

### Option 1 : Installateur Automatique

1. **Téléchargez** `SpotlightWindows-Setup.exe`

2. **Exécutez** l'installateur (double-clic)

3. **Suivez** l'assistant d'installation :
   - ✅ Acceptez les privilèges administrateur
   - ✅ Choisissez le répertoire (par défaut : `C:\Program Files\Spotlight Windows`)
   - ✅ Cliquez sur "Installer"

4. **C'est terminé !** 🎉
   - L'application démarre automatiquement avec Windows
   - Utilisez **Ctrl+Space** pour ouvrir la recherche

---

## 🎯 Première Utilisation

### Ouvrir Spotlight :
- **Raccourci clavier** : `Ctrl + Space`
- **Icône système** : Clic droit sur l'icône dans la barre des tâches
- **Menu Démarrer** : Spotlight Windows

### Utilisation :
1. Appuyez sur **Ctrl+Space**
2. Tapez ce que vous cherchez (nom de fichier, contenu...)
3. Utilisez les **flèches ↑↓** pour naviguer
4. Appuyez sur **Enter** pour ouvrir
5. Appuyez sur **ESC** pour fermer

### Indexation Initiale :
- Au premier lancement, Spotlight indexe vos fichiers
- Cela peut prendre 1-2 minutes selon la quantité de fichiers
- La recherche devient **quasi-instantanée** après l'indexation
- L'index est persistant (pas de réindexation au redémarrage)

---

## ⚙️ Configuration

### Répertoires indexés par défaut :
- ✅ Votre profil utilisateur (`C:\Users\VotreNom\`)
- ✅ Disques D:, E:, F: (s'ils existent)
- ✅ Program Files

### Répertoires exclus (pour performance) :
- ❌ `C:\Windows`
- ❌ `node_modules`
- ❌ Fichiers temporaires et caches

---

## 🔧 Option 2 : Installation Manuelle (Portable)

Si vous ne voulez pas utiliser l'installateur :

1. **Téléchargez** `spotlight_windows.exe`

2. **Copiez-le** où vous voulez (ex: `C:\Tools\`)

3. **Lancez-le** (double-clic)

4. **(Optionnel) Démarrage automatique** :
   - Appuyez sur `Win+R`
   - Tapez `shell:startup`
   - Créez un raccourci vers `spotlight_windows.exe`

---

## 🗑️ Désinstallation

### Méthode 1 : Panneau de Configuration (Recommandé)
1. Ouvrez **Paramètres Windows** > **Applications**
2. Recherchez **"Spotlight Windows"**
3. Cliquez sur **Désinstaller**

### Méthode 2 : Désinstalleur Direct
1. Allez dans `C:\Program Files\Spotlight Windows\`
2. Lancez `Uninstall.exe`

### Méthode 3 : Menu Démarrer
1. Menu Démarrer > **Spotlight Windows**
2. Cliquez sur **Désinstaller**

**Note** : Vos données d'index sont conservées dans `%APPDATA%\spotlight_windows\` pour une éventuelle réinstallation. Supprimez manuellement ce dossier si vous voulez tout effacer.

---

## ❓ Dépannage

### Spotlight ne s'ouvre pas avec Ctrl+Space
- **Cause** : Le hotkey n'est pas enregistré
- **Solution** : Relancez l'application depuis le Menu Démarrer

### L'application ne démarre pas automatiquement
- **Cause** : Désactivé dans les paramètres de démarrage
- **Solution** :
  1. Ouvrez **Gestionnaire des tâches** (Ctrl+Shift+Esc)
  2. Onglet **Démarrage**
  3. Activez **Spotlight Windows**

### Recherche lente ou aucun résultat
- **Cause** : Indexation en cours ou incomplète
- **Solution** :
  1. Attendez 2-3 minutes après le premier lancement
  2. Vérifiez l'icône système (tooltip indique l'état)

### L'application plante ou se ferme
- **Cause** : Conflit avec antivirus ou permissions
- **Solution** :
  1. Ajoutez `spotlight_windows.exe` aux exceptions de l'antivirus
  2. Lancez en tant qu'administrateur

### Fichiers manquants dans les résultats
- **Cause** : Fichier dans un répertoire exclu
- **Solution** : Les répertoires système et caches sont exclus par défaut pour la performance

---

## 📊 Informations Système

### Configuration Requise :
- 💻 **OS** : Windows 10/11 (64-bit)
- 💾 **RAM** : 4 GB minimum (8 GB recommandé)
- 💿 **Disque** : 50 MB pour l'application + espace pour l'index
- 🖥️ **Processeur** : Dual-core ou supérieur

### Taille de l'Index :
- Dépend du nombre de fichiers
- ~100 MB pour 100,000 fichiers
- Stocké dans `%APPDATA%\spotlight_windows\index\`

### Performance :
- ⚡ Recherche : < 50ms en moyenne
- 🔍 Indexation initiale : 1-5 minutes
- 📂 Surveillance temps réel : Oui (détection automatique nouveaux fichiers)

---

## 🎨 Fonctionnalités

✅ **Recherche ultra-rapide** (Tantivy engine)
✅ **Recherche par contenu** (dans les fichiers texte)
✅ **Recherche floue** (tolère les fautes de frappe)
✅ **N-grams** (trouve "doc" dans "documents")
✅ **Icônes contextuelles** (fichiers, dossiers, types)
✅ **Aperçu du chemin complet**
✅ **Navigation clavier** (flèches, Enter, ESC)
✅ **Thème sombre moderne**
✅ **Animations fluides**
✅ **Système tray** (fonctionne en arrière-plan)
✅ **Démarrage automatique**
✅ **Index persistant** (pas de réindexation)

---

## 📞 Support

### Problèmes ou Questions ?
- 🐛 **Bugs** : [GitHub Issues](https://github.com/fless-lab/spotlight_windows/issues)
- 💬 **Discussions** : [GitHub Discussions](https://github.com/fless-lab/spotlight_windows/discussions)
- 📧 **Email** : support@spotlightwindows.com (si configuré)

### Logs de Débogage :
Les logs sont stockés dans :
```
%APPDATA%\spotlight_windows\logs\
```

Consultez ces fichiers en cas de problème et joignez-les dans votre rapport de bug.

---

## 🎉 Profitez de Spotlight Windows !

Vous êtes prêt à utiliser **Spotlight Windows** !

**Rappel** : Appuyez sur **Ctrl+Space** n'importe quand pour chercher vos fichiers instantanément.

Happy Searching! 🚀
