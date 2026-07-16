# GEVSHOT

Outil de capture d'écran annotée pour documenter des logiciels, écrit en Rust.

Capture → annote → exporte, en moins de 30 secondes.

## Fonctionnalités

- **Capture de région** : `Ctrl+Shift+S` (raccourci global) fige l'écran, on trace
  la zone à la souris. `Entrée` = tout l'écran, `Échap` = annuler.
- **Annotations vectorielles** (éditables : déplacer, supprimer, annuler) :
  - ① pastilles numérotées auto-incrémentées (renumérotation automatique à la suppression)
  - ↗ flèches
  - ▭ rectangles d'encadrement
  - ◯ ellipses d'encerclement
  - couleur (rouge doc par défaut) et épaisseur réglables
- **Export** :
  - `Ctrl+C` → presse-papiers
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
| `Ctrl+S` | Sauver en PNG auto-nommé |
| `Ctrl+Z` / `Ctrl+Y` | Annuler / Rétablir |
| `Suppr` | Supprimer l'annotation sélectionnée |
| `Échap` (sélection) | Annuler la capture |
| `Entrée` (sélection) | Capturer tout l'écran |

## Développement

```powershell
cargo run            # debug
cargo build --release  # binaire optimisé (target/release/Gevshot.exe)
```

Prérequis : Rust (toolchain MSVC) + Visual Studio Build Tools (workload C++).

## Architecture

| Fichier | Rôle |
|---|---|
| `src/main.rs` | Point d'entrée, fenêtre eframe |
| `src/app.rs` | Machine à états : Vide → Sélection de région → Édition |
| `src/capture.rs` | Capture d'écran (xcap) |
| `src/annotation.rs` | Modèle vectoriel des annotations : rendu egui, hit-test |
| `src/editor.rs` | Outils, interactions souris, undo/redo |
| `src/export.rs` | Aplatissement sur pixels (tiny-skia + ab_glyph), presse-papiers, PNG |
| `src/project.rs` | Format `.gev` (JSON : PNG base64 + annotations) |
| `src/tray.rs` | Icône barre système et menu |

Les annotations sont stockées en coordonnées de l'image source et rendues
vectoriellement dans l'éditeur ; elles ne sont aplaties sur les pixels
qu'au moment de l'export. Voir `SPEC.md` pour la spécification complète.
