; ===================================
; Spotlight Windows - Installateur NSIS Professionnel
; ===================================
; Installez NSIS depuis https://nsis.sourceforge.io/
; Compilez avec: makensis installer.nsi

!include "MUI2.nsh"
!include "FileFunc.nsh"

; ===================================
; Configuration Générale
; ===================================
Name "Spotlight Windows"
OutFile "SpotlightWindows-Setup.exe"
Unicode True

; Répertoire d'installation par défaut
InstallDir "$PROGRAMFILES64\Spotlight Windows"

; Demander les privilèges administrateur
RequestExecutionLevel admin

; Compression maximale
SetCompressor /SOLID lzma
SetCompressorDictSize 64

; Version de l'application
!define VERSION "0.3.0"
!define COMPANY "Spotlight Windows Team"
!define PRODUCT_NAME "Spotlight Windows"
!define PRODUCT_EXE "spotlight_windows.exe"

; Registry keys
!define REGKEY "Software\${COMPANY}\${PRODUCT_NAME}"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"

; ===================================
; Interface Modern UI 2
; ===================================
!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install-blue.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall-blue.ico"

!define MUI_WELCOMEFINISHPAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Wizard\nsis3-branding.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Wizard\nsis3-branding.bmp"

; Page de bienvenue
!define MUI_WELCOMEPAGE_TITLE "Bienvenue dans l'installation de Spotlight Windows"
!define MUI_WELCOMEPAGE_TEXT "Cet assistant va vous guider dans l'installation de Spotlight Windows.$\r$\n$\r$\nSpotlight Windows est un outil de recherche ultra-rapide pour Windows, inspiré de macOS Spotlight.$\r$\n$\r$\nCliquez sur Suivant pour continuer."
!insertmacro MUI_PAGE_WELCOME

; Page de licence (optionnel)
; !insertmacro MUI_PAGE_LICENSE "LICENSE.txt"

; Page de sélection du répertoire
!insertmacro MUI_PAGE_DIRECTORY

; Page d'installation
!insertmacro MUI_PAGE_INSTFILES

; Page de fin avec option de lancement
!define MUI_FINISHPAGE_TITLE "Installation terminée !"
!define MUI_FINISHPAGE_TEXT "Spotlight Windows a été installé avec succès.$\r$\n$\r$\nL'application démarrera automatiquement avec Windows.$\r$\n$\r$\nUtilisez Ctrl+Space pour ouvrir la fenêtre de recherche."
!define MUI_FINISHPAGE_RUN "$INSTDIR\${PRODUCT_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Lancer Spotlight Windows maintenant"
!insertmacro MUI_PAGE_FINISH

; Pages de désinstallation
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Langues
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "English"

; ===================================
; Métadonnées du fichier
; ===================================
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey "CompanyName" "${COMPANY}"
VIAddVersionKey "FileDescription" "Installateur Spotlight Windows"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "© 2025 ${COMPANY}"

; ===================================
; Section d'installation
; ===================================
Section "Installation" SecInstall
  SetOutPath "$INSTDIR"

  ; Vérifier si l'application est en cours d'exécution
  nsExec::ExecToLog 'taskkill /F /IM ${PRODUCT_EXE}'
  Sleep 1000

  ; Copier les fichiers
  File "target\x86_64-pc-windows-gnu\release\${PRODUCT_EXE}"

  ; Créer le répertoire de données
  CreateDirectory "$APPDATA\spotlight_windows"

  ; Écrire les informations dans le registre
  WriteRegStr HKLM "${REGKEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${REGKEY}" "Version" "${VERSION}"

  ; Ajouter au démarrage automatique Windows
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCT_NAME}" '"$INSTDIR\${PRODUCT_EXE}"'

  ; Créer le raccourci dans le menu Démarrer
  CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
  CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_EXE}"
  CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\Désinstaller.lnk" "$INSTDIR\Uninstall.exe"

  ; Créer le raccourci sur le bureau (optionnel)
  CreateShortcut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_EXE}"

  ; Créer le désinstalleur
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Ajouter l'entrée dans Ajout/Suppression de programmes
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "Publisher" "${COMPANY}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "HelpLink" "https://github.com/fless-lab/spotlight_windows"
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair" 1

  ; Calculer la taille installée
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "EstimatedSize" "$0"

  DetailPrint "Installation terminée avec succès !"
SectionEnd

; ===================================
; Section de désinstallation
; ===================================
Section "Uninstall"
  ; Arrêter l'application si elle tourne
  nsExec::ExecToLog 'taskkill /F /IM ${PRODUCT_EXE}'
  Sleep 1000

  ; Supprimer du démarrage automatique
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCT_NAME}"

  ; Supprimer les fichiers
  Delete "$INSTDIR\${PRODUCT_EXE}"
  Delete "$INSTDIR\Uninstall.exe"

  ; Supprimer les raccourcis
  Delete "$DESKTOP\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${PRODUCT_NAME}\Désinstaller.lnk"
  RMDir "$SMPROGRAMS\${PRODUCT_NAME}"

  ; Supprimer le répertoire d'installation
  RMDir "$INSTDIR"

  ; NE PAS supprimer les données utilisateur (index, config)
  ; L'utilisateur peut les garder pour une réinstallation
  ; Si vous voulez les supprimer, décommentez :
  ; RMDir /r "$APPDATA\spotlight_windows"

  ; Supprimer les clés de registre
  DeleteRegKey HKLM "${REGKEY}"
  DeleteRegKey HKLM "${UNINSTALL_KEY}"

  DetailPrint "Désinstallation terminée !"
SectionEnd

; ===================================
; Fonction d'initialisation
; ===================================
Function .onInit
  ; Vérifier si déjà installé
  ReadRegStr $0 HKLM "${UNINSTALL_KEY}" "UninstallString"
  ${If} $0 != ""
    MessageBox MB_YESNO|MB_ICONQUESTION \
      "${PRODUCT_NAME} est déjà installé. Voulez-vous désinstaller la version existante ?" \
      IDYES uninst
    Abort

    uninst:
      ExecWait '"$0" /S _?=$INSTDIR'
      Delete "$0"
  ${EndIf}
FunctionEnd

; ===================================
; Messages de description
; ===================================
LangString DESC_SecInstall ${LANG_FRENCH} "Installe Spotlight Windows sur votre ordinateur."
LangString DESC_SecInstall ${LANG_ENGLISH} "Installs Spotlight Windows on your computer."

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecInstall} $(DESC_SecInstall)
!insertmacro MUI_FUNCTION_DESCRIPTION_END
