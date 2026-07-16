# Gevshot — Spécification v1

Outil de capture d'écran orienté **documentation logicielle**, écrit en Rust.
Boucle cible : capturer → annoter → exporter, en moins de 30 secondes.

## Décisions de cadrage (brainstorming 2026-07-16)

| Sujet | Décision |
|---|---|
| Mode de vie | Icône barre système + raccourci global (Ctrl+Shift+S) ; l'éditeur ne s'ouvre qu'après une capture |
| Capture v1 | Région à la souris (overlay plein écran, rectangle de sélection) |
| Annotations v1 | Pastilles numérotées, flèches, rectangles, ellipses |
| Édition | Annotations vectorielles éditables (déplacer, supprimer, modifier) + format projet sauvegardable |
| Export | Presse-papiers, PNG nommage auto (`capture-AAAA-MM-JJ-NNN.png`), PNG « Enregistrer sous » |
| Plateforme | Windows 11 d'abord (bibliothèques portables pour plus tard) |
| Style | Rouge doc classique par défaut, couleur + épaisseur modifiables |

## Fonctionnement

1. L'appli démarre dans la barre système (pas de fenêtre).
2. `Ctrl+Shift+S` (ou clic sur l'icône → « Capturer ») déclenche la capture :
   - capture de l'écran complet en mémoire ;
   - overlay plein écran assombri, l'utilisateur trace un rectangle ;
   - la zone sélectionnée devient l'image de travail.
3. L'éditeur s'ouvre avec l'image et la barre d'outils :
   - **Sélection** : cliquer une annotation pour la déplacer/supprimer (Suppr).
   - **Pastille numérotée** : clic → cercle numéroté auto-incrémenté ; renumérotation automatique en cas de suppression.
   - **Flèche** : glisser du départ vers la cible, tête à l'arrivée.
   - **Rectangle** : glisser pour encadrer.
   - **Ellipse** : glisser pour encercler.
   - **Texte** : étiquette libre (police, taille, gras). *(ajouté post-v1)*
   - **Recadrer** : glisser la zone à conserver, Entrée pour valider ;
     les annotations hors zone sont supprimées, pastilles renumérotées. *(ajouté post-v1)*
   - Couleur (rouge par défaut) + épaisseur du trait.
   - Undo/Redo (Ctrl+Z / Ctrl+Y).
4. Export :
   - `Ctrl+C` → image annotée dans le presse-papiers ;
   - bouton « Sauver » → PNG auto-nommé dans le dossier configuré ;
   - « Enregistrer sous… » → dialogue classique ;
   - « Sauver le projet » → fichier `.gev` (JSON : image source en base64/PNG + liste des annotations) ré-ouvrable pour régénérer la capture quand le logiciel documenté évolue.

## Architecture

- **Annotations vectorielles** : `enum Annotation { Badge, Arrow, Rect, Ellipse }`,
  coordonnées en pixels de l'image source. Rendues par-dessus la texture dans
  l'éditeur ; « aplaties » sur les pixels uniquement à l'export.
- **Undo/Redo** : pile d'états de la liste d'annotations (elle reste petite).
- **Format projet** : `serde` → JSON (`.gev`), image embarquée en PNG base64.

## Crates

| Rôle | Crate |
|---|---|
| GUI + canvas | `eframe` / `egui` |
| Capture d'écran | `xcap` |
| Barre système | `tray-icon` |
| Raccourci global | `global-hotkey` |
| Presse-papiers image | `arboard` |
| Export PNG / dessin | `image` + `imageproc` |
| Dialogue fichier | `rfd` |
| Sérialisation projet | `serde` + `serde_json` + `base64` |

## Hors périmètre v1 (candidats v2)

- ~~Texte libre~~ *(livré)*, flou/pixelisation, surligneur
- ~~Capture fenêtre active, capture différée (minuteur)~~ *(livré)*
- Multi-écrans avancé (v1 : écran principal)
- Historique/galerie des captures
- ~~Linux/macOS~~ *(bundles .deb / .app construits par la CI depuis la 1.0.1)*

## Jalons

1. **M1 — Squelette** : fenêtre eframe, capture plein écran via xcap affichée dans l'éditeur.
2. **M2 — Annotations** : les 4 outils + sélection/déplacement/suppression + undo/redo.
3. **M3 — Export** : aplatissement imageproc, presse-papiers, PNG auto + dialogue.
4. **M4 — Capture région** : overlay de sélection.
5. **M5 — Intégration Windows** : barre système, raccourci global, démarrage silencieux.
6. **M6 — Format projet** : sauvegarde/ouverture `.gev`.
