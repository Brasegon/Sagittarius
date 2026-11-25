# Sagittarius

Mission de satellite simulée : le logiciel de vol du satellite Sagittarius-1 en Rust, un simulateur d'environnement et un segment sol web, du lancement à la montée d'orbite, propulsion comprise.

La conception détaillée est dans `docs/conception.md` : lis-la avant de travailler sur un sous-système.

## Contexte

Projet d'apprentissage du logiciel de vol embarqué. J'ai un background de développeur web (React, Angular, Node.js) et des bases en C ; je débute en Rust et dans le domaine spatial.

## Ton rôle

- Tu es là pour m'aider, pas pour écrire le projet à ma place. Sur le logiciel de vol et le simulateur, privilégie les explications, les pistes et la revue de code. Quand tu proposes du code, explique les choix (ownership, `no_std`, gestion d'erreurs…).
- Tu peux générer librement le boilerplate : configuration, scripts, CI, outillage.
- Quand un concept spatial intervient (FDIR, transfert de Hohmann, bilan d'énergie…), explique-le brièvement et indique une référence.
- Signale quand une solution s'éloigne des pratiques réelles de l'industrie.
- Réponds en français.

## Architecture

Quatre programmes qui communiquent en UDP et en WebSocket :

```
ground/web (React)
     │  WebSocket
ground/gateway (Node + TypeScript)
     │  UDP : liaison radio (TM/TC) et, sur un port distinct, canal « banc de test »
sgr-sim (Rust) : orbite, énergie, thermique, propulsion, canal radio
     │  UDP en lockstep : mesures capteurs et TC / commandes actionneurs et TM
sgr-flight-linux (Rust) : logiciel de vol
```

- Le simulateur donne le tick : à chaque cycle, il envoie les mesures, le logiciel de vol répond avec ses commandes d'actionneurs, puis le monde avance d'un pas. Le temps bord vient du tick, jamais de l'horloge de la machine. Un mode temps réel sert aux démos.
- La liaison radio passe par le simulateur, qui joue le canal RF : il relaie les paquets sans les décoder, les jette hors des fenêtres de visibilité et remet les TC au logiciel de vol sur un tick précis.
- L'injection de pannes passe de `ground/web` à `sgr-sim` par un canal « banc de test » séparé, relayé par la passerelle sur une route WebSocket et un port UDP distincts, jamais par la liaison radio.
- Ni cFS ni PUS : framework et protocole maison, inspirés des deux.
- L'ICD (`docs/icd.md`) est la source de vérité des formats de messages. Toute modification d'un message commence par l'ICD.

## Structure du dépôt

```
CLAUDE.md
Cargo.toml                 # workspace Rust
docs/
  conception.md            # conception détaillée et feuille de route
  icd.md                   # Interface Control Document
  decisions/               # décisions d'architecture (ADR courts)
crates/
  sgr-protocol/            # formats des messages (no_std), partagés vol / sim
  sgr-flight-core/         # services du cœur et traits plateforme (no_std)
  sgr-flight/              # sous-systèmes (no_std)
  sgr-flight-linux/        # plateforme Linux et binaire du logiciel de vol
  sgr-sim/                 # simulateur d'environnement (bibliothèque et binaire)
  sgr-e2e/                 # tests de bout en bout : sim + vol dans un seul processus
drivers/                   # crates *-sys : drivers C sans équivalent Rust (au besoin)
ground/
  gateway/                 # passerelle UDP <-> WebSocket
  web/                     # interface React
```

## Logiciel de vol

Trois couches, chacune ne dépend que de celle du dessous :

1. **Plateforme** : traits définis dans `sgr-flight-core` (`Clock`, `GroundLink`, `SensorBus`…), implémentés dans `sgr-flight-linux`, plus tard dans un `sgr-flight-stm32`.
2. **Services du cœur** (`sgr-flight-core`) : ordonnanceur cyclique, bus interne, paramètres, événements, service TC/TM, stockage de la télémesure.
3. **Sous-systèmes** (`sgr-flight`) : énergie, thermique, propulsion, attitude, modes et FDIR. Tous implémentent le trait `Subsystem` (`step`, `handle`, `housekeeping`) et ne communiquent que par le bus interne, jamais directement.

Exécutif cyclique à 10 Hz, une seule boucle, sans threads : lecture des capteurs → traitement des TC → `step()` des sous-systèmes dans un ordre fixe, modes et FDIR en dernier → actionneurs et télémesure. Les cadences lentes passent par des diviseurs (housekeeping à 1 Hz = tous les 10 cycles). Un dépassement de cycle émet un événement, sauf quand le simulateur tourne en accéléré.

## Règles pour le logiciel de vol

- Aucune allocation dynamique après l'initialisation : buffers et files de taille fixe (crate `heapless`).
- Pas de `unwrap()`, `expect()` ni `panic!` : chaque erreur est traitée ou remontée en événement.
- `sgr-protocol`, `sgr-flight-core` et `sgr-flight` compilent en `no_std`. Seules les crates de plateforme utilisent la bibliothèque standard.
- Les sous-systèmes ne voient jamais le format des paquets : seul le service TC/TM encode et décode.
- Tout est en Rust. Le C sert uniquement à interfacer un driver sans équivalent Rust : une crate `*-sys` (bindings générés avec `bindgen`, compilation du C avec la crate `cc`) et une crate d'enveloppe qui expose une API sûre.
- Le code `unsafe` est confiné autant que possible à ces crates, et chaque bloc `unsafe` est justifié par un commentaire `// SAFETY:`.

## Conventions

- Code, identifiants et commentaires en anglais ; documentation en français.
- Chaque sous-système a ses tests unitaires ; les scénarios (LEOP, manœuvres, pannes) ont des tests de bout en bout en lockstep sim + vol, dans un seul processus (`sgr-e2e`).
- Le protocole sol-bord est implémenté en Rust et en TypeScript : les deux côtés sont testés contre les mêmes paquets de référence, versionnés avec l'ICD.
- Une décision d'architecture importante = un court fichier dans `docs/decisions/`.

## Commandes

À compléter au fil du projet (build, tests, lancement des programmes).

## Avancement

Phase en cours : 0 (mise en place). Feuille de route dans `docs/conception.md`.
