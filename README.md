# GEVSHOT

Outil de capture d'écran annotée pour documenter des logiciels, écrit en Rust.

Capture → annote → exporte, en moins de 30 secondes.

## Fonctionnalités

- **Capture de région** : `Ctrl+Shift+S` (raccourci global) fige l'écran, on trace
  la zone à la souris. `Entrée` = tout l'écran, `Échap` = annuler. Une barre en
  haut de l'overlay permet de changer de mode sans annuler : région, fenêtre
  au choix, écran entier, région différée (3 s). L'image est copiée dans le
  presse-papiers dès la capture.
- **Annotations vectorielles** (éditables : déplacer, supprimer, annuler) :
  - ① pastilles numérotées auto-incrémentées (renumérotation automatique à la suppression)
  - ↗ flèches
  - ▭ rectangles d'encadrement
  - ◯ ellipses d'encerclement
  - 🗛 étiquettes de texte (police, taille, gras)
  - couleur (rouge doc par défaut) et épaisseur réglables
- **Recadrage** : tracez la zone à conserver, `Entrée` pour valider — les
  annotations suivent l'image, celles hors zone sont supprimées et les
  pastilles renumérotées.
- **Export** :
  - copie automatique dans le presse-papiers après chaque capture ; `Ctrl+C`
    pour recopier avec les annotations à jour
  - `Ctrl+S` → PNG auto-nommé `capture-AAAA-MM-JJ-NNN.png` dans `Images\Gevshot`
  - « Enregistrer sous… » classique
- **Format projet `.gev`** : image + annotations rouvrables. Quand le logiciel
  documenté évolue, « 🔄 Recapturer le fond » remplace l'image en conservant
  les annotations.
- **Barre système** : l'appli démarre silencieusement, fermer la fenêtre la
  masque, tout passe par l'icône ou le raccourci global.

## Raccourcis

| Raccourci | Action |
|---|---|
| `Ctrl+Shift+S` | Capturer (global, même appli masquée) |
| `Ctrl+C` | Copier l'image annotée |
| `Ctrl+S` | Exporter en PNG auto-nommé |
| `Ctrl+Z` / `Ctrl+Y` | Annuler / Rétablir |
| `Suppr` | Supprimer l'annotation sélectionnée |
| `Échap` (sélection) | Annuler la capture |
| `Entrée` (sélection) | Capturer tout l'écran |
| `Entrée` / `Échap` (recadrage) | Valider / annuler le recadrage |
| `Ctrl+molette` | Zoomer (ancré sur le curseur) |
| Molette / glisser bouton milieu | Déplacer la vue zoomée |
| `Ctrl+0` | Réajuster l'image à la fenêtre |

## Installation

Sur la [page des releases](https://github.com/Gnandal/gevshot/releases/latest) :

| Plateforme | Fichiers |
|---|---|
| Windows 10/11 | `GEVSHOT-<version>-x64.msi` (installeur) ou `gevshot-…-windows-x64.zip` (exe portable) |
| Ubuntu / Debian | `gevshot_<version>_amd64.deb` (entrée de menu incluse) ou `gevshot-…-linux-x64.tar.gz` |
| macOS 11+ | `gevshot-…-macos-universal.zip` (app universelle Intel + Apple Silicon) |

L'app macOS n'est pas signée : au premier lancement, clic droit → « Ouvrir »,
puis autorisez l'enregistrement de l'écran. Sous Linux, X11 est recommandé
(Wayland selon le compositeur).

## Développement

```powershell
cargo run              # debug
cargo build --release  # binaire optimisé
```

Prérequis :

- **Windows** : Rust (toolchain MSVC) + Visual Studio Build Tools (workload C++).
- **Linux** : `libgtk-3-dev libayatana-appindicator3-dev libxkbcommon-dev libxcb1-dev libdbus-1-dev pkg-config`.
- **macOS** : Xcode Command Line Tools.

## Architecture

Le code est organisé en quatre couches :

| Couche | Contenu |
|---|---|
| `src/domain/` | Métier pur : `annotation.rs` (modèle vectoriel, rendu egui, hit-test), `editor.rs` (état de l'éditeur, outils, undo/redo, légende) |
| `src/infra/` | Système et réseau : `capture.rs` (xcap), `export.rs` (aplatissement tiny-skia + ab_glyph, presse-papiers, PNG), `project.rs` (format `.gev`), `config.rs`, `fonts.rs`, `tray.rs`, `ai.rs` (API Claude) |
| `src/ui/` | Présentation réutilisable : `theme.rs`, `widgets.rs` (boutons à icônes peintes), `settings.rs` (contenu des Paramètres) |
| `src/app/` | Orchestration : `mod.rs` (état global, cycle de vie eframe), puis un module par aspect — `capture_flow`, `region_select`, `toolbar`, `status_bar`, `legend`, `window_picker`, `ai_flow`, `files`, `settings_modal` |

`src/main.rs` assemble le tout ; `src/appicon.rs` (icône dessinée par code)
est partagé avec `build.rs`, qui génère le `.ico` embarqué dans l'exe.

Les annotations sont stockées en coordonnées de l'image source et rendues
vectoriellement dans l'éditeur ; elles ne sont aplaties sur les pixels
qu'au moment de l'export. Voir `SPEC.md` pour la spécification complète.
