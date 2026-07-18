# Cahier des charges — Design de GEVSTUDIO

**Destinataire :** graphiste / UI designer
**Version :** 1.0 — 18/07/2026
**Contact projet :** (à compléter)

---

## 1. Contexte & objectif

**GEVSTUDIO** est une application **bureau** (Windows en priorité, puis Linux et
macOS) qui transforme une suite de **captures d'écran annotées** en un **guide
pas-à-pas** exportable en **PDF** et **PowerPoint (PPTX)**. C'est l'outil de
documentation qui complète **GEVSHOT** (l'outil de capture + annotation déjà
existant).

Le fonctionnement : l'utilisateur importe des captures (chacune est une
« étape »), leur ajoute un **titre** et une **légende**, réordonne les étapes,
puis **exporte** un document propre et homogène.

> **Objectif de la mission :** doter GEVSTUDIO d'une **identité visuelle
> professionnelle** et d'une **interface soignée, moderne et rassurante**, digne
> d'un logiciel payant. Le produit sera **monétisé** (voir §7), l'exigence de
> finition est donc élevée.

**Ce qui existe déjà (à améliorer, pas à refaire de zéro) :** l'application est
fonctionnelle avec une interface provisoire (thème lavande clair). Une capture
de l'état actuel est fournie en annexe (§10). Le rôle du graphiste est de
**professionnaliser** cet existant, pas de tout réinventer.

---

## 2. Positionnement & ton visuel

- **Cible :** créateurs de documentation, formateurs, équipes support et
  produit, freelances qui rédigent des tutoriels.
- **Promesse :** « De la capture au guide fini en quelques minutes. »
- **Personnalité de marque :** **clair, net, efficace, fiable**. Un outil
  « pro » qui ne fait pas peur — proche d'outils comme Snagit, Scribe ou
  CleanShot, mais plus sobre.
- **Ce qu'on veut éviter :** le côté gadget, les dégradés criards, le « fun »
  excessif, la surcharge d'icônes.
- **Mots-clés d'ambiance :** épuré · lumineux · structuré · confiance.

---

## 3. Périmètre de la mission

Le graphiste doit livrer :

1. **Identité visuelle** — logo GEVSTUDIO, icône d'application, système de
   couleurs, typographie (§4).
2. **Jeu d'icônes** cohérent pour la barre d'outils et la timeline (§4.5).
3. **Maquettes des écrans** de l'application (§5).
4. **Spécifications des composants UI** (boutons, champs, listes, panneaux) (§6).
5. **Écrans liés à la monétisation** (licence / achat / états Free vs Pro) (§7).
6. **Kit de livraison** : fichier source (Figma de préférence), assets exportés,
   mini guide de style (§8).

---

## 4. Identité visuelle

### 4.1 Logo

- Créer un **logotype GEVSTUDIO** (nom + éventuel symbole) et un **symbole seul**
  (monogramme / pictogramme) utilisable en petit.
- **Cohérence de gamme :** GEVSTUDIO fait partie d'une famille avec **GEVSHOT**.
  Proposer un système où les deux se ressemblent (même symbole décliné, ou même
  construction typographique) tout en étant distinguables. GEVSHOT = capture,
  GEVSTUDIO = assemblage/document.
- Décliner le logo en **3 versions** : couleur (fond clair), monochrome sombre,
  monochrome blanc (fond foncé).

### 4.2 Icône d'application

Icône du logiciel (barre des tâches, raccourci bureau, fenêtre).

- Lisible et reconnaissable de **16 px à 512 px**.
- À livrer aux tailles : **16, 32, 48, 64, 128, 256, 512, 1024 px**.
- Formats de sortie : **PNG** (toutes tailles), **.ico** (Windows, multi-tailles
  dans un seul fichier), **.icns** (macOS).
- Contrainte : l'icône doit rester claire en très petit (16/32 px) — éviter les
  détails fins.

### 4.3 Palette de couleurs

Une base existe (thème « lavande clair »). Le graphiste peut la **conserver et
l'affiner** ou proposer une évolution justifiée. Valeurs actuelles :

| Rôle | Couleur | Hex |
|------|---------|-----|
| Fond panneaux / barres | lavande très clair | `#F6F4FB` |
| Fond zone d'aperçu | gris-lavande | `#ECEAF4` |
| Survol | lavande | `#EAE7F3` |
| Accent principal (actions, sélection) | bleu | `#1D4ED8` |
| Accent doux (fonds sélectionnés) | bleu clair | `#D9E6FA` |
| Texte principal | gris très foncé | `#23262E` |
| Texte secondaire | gris moyen | `#6E727D` |
| Bordures | gris-lavande clair | `#DDDAE7` |
| Succès | vert | `#1E9A4E` |
| Danger / suppression | rouge | `#C6302C` |

**Couleurs d'annotation** (utilisées dans les captures, à ne PAS réattribuer à
l'UI pour éviter la confusion) : rouge `#C6302C`, bleu `#1A73E8`, orange
`#F59117`, vert `#22A34F`, noir `#17181C`.

Exigences : contraste **WCAG AA** minimum pour le texte ; prévoir les **états**
(normal / survol / actif / désactivé) de chaque couleur d'action. *(Un mode
sombre n'est pas demandé pour la v1, mais une proposition est bienvenue en
option.)*

### 4.4 Typographie

- L'interface utilise actuellement les polices système. Le graphiste **spécifie
  une police d'interface** (idéalement libre / open-source pour éviter les
  licences : ex. Inter, Source Sans, IBM Plex Sans).
- Définir une **échelle typographique** : titre de section, libellé, corps,
  légende, petit texte de statut. Indiquer tailles, graisses et interlignes.
- Note technique : les **exports** (PDF/PPTX) utilisent la police **DejaVu Sans**
  embarquée pour un rendu identique partout — la typo d'interface et la typo
  d'export peuvent différer, ce n'est pas gênant.

### 4.5 Iconographie (jeu d'icônes)

Créer un **jeu d'icônes homogène** (même grille, même épaisseur de trait, même
style — plutôt **linéaire / outline**). Icônes nécessaires au minimum :

- Ouvrir un fichier
- Enregistrer / Enregistrer sous
- Ajouter une étape depuis une image
- Ajouter une étape depuis une capture GEVSHOT (.gev)
- Exporter en PDF
- Exporter en PowerPoint (PPTX)
- Exporter en vidéo/GIF *(fonction à venir — prévoir l'icône)*
- Monter / Descendre une étape (réordonner)
- Supprimer une étape
- Paramètres
- Aide / À propos
- Badge « Pro » / cadenas (verrou de fonctionnalité, voir §7)

Livrer en **SVG** + **PNG @1x et @2x** (voir contrainte technique §8).

---

## 5. Écrans à concevoir

L'application est une **fenêtre unique** structurée en zones. Maquetter chaque
zone et l'assemblage complet (résolution de référence **1280 × 800**, la fenêtre
étant redimensionnable).

### 5.1 Fenêtre principale (écran central du produit)

Disposition actuelle à professionnaliser :

```
┌──────────────────────────────────────────────────────────────────┐
│  BARRE D'OUTILS (haut)                                            │
│  [Ouvrir] [Enregistrer] [Enreg. sous] | [+ Image] [+ .gev]       │
│  | [Exporter PDF] [Exporter PPTX]      Titre du guide : [______]  │
├───────────────┬──────────────────────────────────────────────────┤
│  TIMELINE      │  ZONE CENTRALE                                   │
│  (panneau      │  Titre de l'étape : [__________________]         │
│   gauche)      │  ┌────────────────────────────────────────────┐  │
│                │  │                                            │  │
│  Étapes (N)    │  │     APERÇU de la capture annotée           │  │
│  1. Titre  ◀── │  │     (centré, mis à l'échelle)              │  │
│    [Monter]    │  │                                            │  │
│    [Descendre] │  └────────────────────────────────────────────┘  │
│    [Supprimer] │  Légende : [______________________________]      │
│  2. (sans …)   │                                                  │
├───────────────┴──────────────────────────────────────────────────┤
│  BARRE DE STATUT (bas) : messages (« PDF exporté : … »)          │
└──────────────────────────────────────────────────────────────────┘
```

Zones à traiter :

- **Barre d'outils (haut)** : groupes d'actions clairement séparés (fichier /
  ajout d'étape / export), + champ « Titre du guide ». Proposer icônes +
  libellés. Gérer le cas où la fenêtre est étroite (débordement).
- **Timeline (gauche, redimensionnable)** : liste ordonnée des étapes.
  Concevoir l'**élément d'étape** : numéro, **vignette miniature** de la capture
  (idéalement), titre, et actions (monter / descendre / supprimer) sur l'étape
  sélectionnée. Traiter l'**état sélectionné** de façon nette.
- **Zone centrale** : champ « Titre de l'étape » en haut, **grand aperçu** de la
  capture au centre (sur fond neutre), champ « Légende » multi-lignes en bas.
- **Barre de statut (bas)** : zone de message discrète (succès / erreur).

### 5.2 État vide / premier lancement (onboarding)

Écran affiché quand aucun document n'est ouvert / aucune étape. Doit **guider** :
grande zone d'appel à l'action (« Ajoutez une première étape » / « Ouvrez un
document »), illustration légère, boutons primaires. C'est la première impression
— soigner particulièrement.

### 5.3 Dialogue « À propos » / aide

Petite fenêtre : logo, nom, version, lien site, mention licence (Free / Pro).

### 5.4 Boîtes de dialogue système

L'ouverture/enregistrement/export passe par les **dialogues natifs du système**
(pas à redessiner). En revanche, prévoir d'éventuelles **fenêtres modales
internes** : confirmation de suppression, message d'erreur, progression d'export.

---

## 6. Composants UI à spécifier

Fournir les états **normal / survol / actif (pressé) / focus / désactivé** pour
chacun :

- **Boutons** : primaire (action forte, ex. Exporter), secondaire (ex. Ouvrir),
  discret/icône seule (ex. actions de timeline). Rayon des coins actuel : ~8 px.
- **Champs de saisie** : ligne simple (titres) et multi-lignes (légende), avec
  texte d'aide (placeholder).
- **Élément de liste (étape)** : normal / survolé / sélectionné.
- **Séparateurs**, **panneaux**, **barre de statut**.
- **Badge / pastille** « Pro » et **icône cadenas** pour les fonctions
  verrouillées.
- **Info-bulles** (tooltips) au survol des icônes.

Définir la **grille d'espacement** (l'actuel utilise ~8 px d'espacement,
~12×6 px de padding de bouton — à confirmer/harmoniser).

---

## 7. Monétisation — écrans à concevoir

**Modèle retenu :** l'application est **gratuite à l'usage**, avec une **licence
Pro à paiement unique** (one-time, pas d'abonnement) qui débloque une partie des
fonctionnalités. Le déclencheur commercial visé : **au-delà de ~100
utilisateurs**, activer la vente de la licence Pro.

> **Hypothèse de gating** (à valider par le porteur du projet — le graphiste doit
> juste prévoir les écrans) : la version gratuite permet de créer des guides et
> d'exporter en **PDF avec un filigrane** ; la version **Pro** débloque
> l'**export sans filigrane**, le **PPTX** et la **vidéo/GIF** à venir.

Écrans / éléments à concevoir :

1. **Écran / fenêtre « Passer à Pro »** : présentation claire de ce que débloque
   la licence (liste Free vs Pro), prix unique, bouton d'achat, réassurance
   (« paiement unique, à vie »).
2. **Boîte d'activation de licence** : champ pour coller une **clé de licence**,
   bouton « Activer », états succès / erreur.
3. **Indicateurs d'état** : badge **« Pro »** quand la licence est active ;
   **cadenas** + info-bulle sur les boutons verrouillés (ex. « Exporter PPTX —
   réservé à la version Pro »), avec redirection vers l'écran d'achat au clic.
4. **Filigrane** (watermark) pour l'export gratuit : proposer un traitement
   discret mais visible (texte / petit logo en coin de page/diapo).

*(La page web de vente / le paiement en ligne ne font pas partie de cette
mission sauf demande explicite — se concentrer sur les écrans **dans
l'application**.)*

---

## 8. Contraintes techniques (important)

L'application n'est **pas** une page web : elle est développée en **Rust avec
egui** (interface en « mode immédiat »). Conséquences pour le design :

- **Pas de HTML/CSS, pas d'animations complexes** : privilégier des designs
  **nets, plats, à base de formes simples** (aplats, coins arrondis, bordures
  fines). Les ombres portées douces sont possibles mais avec parcimonie.
- **Icônes fournies en bitmap** : livrer les icônes en **PNG @1x et @2x**
  (idéalement 16, 20, 24, 32 px) **en plus** du SVG, car le moteur affiche des
  textures. Fond **transparent**.
- Prévoir les rendus sur **écrans standard et haute densité (HiDPI / ×2)**.
- **Thème clair** pour la v1 (mode sombre en option/bonus).
- Couleurs à fournir en **hex/RGB** ; les icônes monochromes idéalement
  **teintables** (une seule couleur, pour pouvoir les recolorer par état).

---

## 9. Livrables attendus

- **Fichier source éditable** : **Figma** (de préférence) ou équivalent, organisé
  et nommé.
- **Logo** : SVG + PNG (fonds clair/sombre/transparent), 3 déclinaisons.
- **Icône d'application** : PNG (toutes tailles §4.2) + **.ico** + **.icns**.
- **Jeu d'icônes UI** : SVG + PNG @1x/@2x, fond transparent.
- **Maquettes** de tous les écrans du §5 et §7, en **1280×800** (+ variantes
  responsives clés).
- **Mini guide de style** (1–3 pages) : couleurs, typo, espacements, états des
  composants, do/don't.
- Tous les **assets exportés** dans une arborescence claire, prêts à intégrer.

---

## 10. Annexes

- **Capture de l'état actuel de l'interface** : `gevstudio_ui_actuel.png`
  (fournie séparément) — sert de base « avant » à professionnaliser.
- **Exemples d'exports** produits par l'outil : un PDF et un PPTX de démonstration
  (fournis séparément) pour comprendre le rendu final visé.
- Références d'inspiration (à discuter) : Snagit, Scribe, CleanShot X, Guidde.

---

## 11. Jalons proposés (à ajuster)

1. **J+3** : pistes de logo + direction visuelle (moodboard, 2–3 directions).
2. **J+7** : identité validée (logo, couleurs, typo, icône d'app).
3. **J+14** : maquettes de la fenêtre principale + état vide.
4. **J+18** : écrans monétisation + jeu d'icônes complet.
5. **J+21** : livraison finale + guide de style + assets exportés.

---

*Questions ouvertes à trancher avec le porteur du projet : nom commercial
définitif, prix de la licence Pro, périmètre exact du gating Free/Pro, présence
ou non d'un mode sombre en v1.*
