# Guide du projet Sagittarius

Ce guide sert à comprendre le projet en partant de zéro, et à le reprendre après une pause. Il explique ce qui existe, pourquoi c'est fait ainsi, et où trouver le reste.

> **État décrit : 2026-11-26**, branche `claude/icd-v0-lockstep`, phase 1. Les sections [3](#3-où-en-est-le-projet) et [13](#13-les-tests) vieillissent vite : voir [Tenir ce guide à jour](#21-tenir-ce-guide-à-jour).

## Comment lire ce guide

Pour reprendre après une pause, lire dans l'ordre les sections 1, 2 et 3, puis la 7, qui suit une mesure à travers tout le code. Le reste se consulte au besoin.

Chaque document du dépôt a un rôle :

| Document | Ce qu'on y trouve | Quand le lire |
| -------- | ----------------- | ------------- |
| Ce guide | Une vue d'ensemble expliquée, et l'état du projet | Pour comprendre ou reprendre |
| [`conception.md`](conception.md) | La cible : mission, architecture visée, feuille de route | Avant de travailler sur un sous-système |
| [`icd.md`](icd.md) | Le format de chaque message, octet par octet | Avant de toucher à un message |
| [`decisions/`](decisions/) | Une décision d'architecture par fichier, avec ses raisons | Pour savoir pourquoi un choix a été fait |
| [`en-cours.md`](en-cours.md) | L'étape en cours, dans le détail. Note temporaire | En premier, s'il existe |
| [`../CLAUDE.md`](../CLAUDE.md) | Les règles et conventions du projet, pour l'assistant comme pour l'auteur | Pour retrouver une règle |

Ce guide résume les autres documents. L'ICD fait foi pour les formats, les décisions pour les choix, et le code pour ce qui existe vraiment. Si le guide contredit l'un d'eux, c'est le guide qui est en retard.

## Sommaire

1. [Le projet en une page](#1-le-projet-en-une-page)
2. [Reprendre en dix minutes](#2-reprendre-en-dix-minutes)
3. [Où en est le projet](#3-où-en-est-le-projet)
4. [Vue d'ensemble : quatre programmes](#4-vue-densemble--quatre-programmes)
5. [Le cœur du projet : le cycle en lockstep](#5-le-cœur-du-projet--le-cycle-en-lockstep)
6. [Lire des octets](#6-lire-des-octets)
7. [Suivre une température de bout en bout](#7-suivre-une-température-de-bout-en-bout)
8. [Le dépôt, dossier par dossier](#8-le-dépôt-dossier-par-dossier)
9. [Le logiciel de vol](#9-le-logiciel-de-vol)
10. [Le simulateur](#10-le-simulateur)
11. [Le segment sol](#11-le-segment-sol)
12. [Les règles du logiciel de vol](#12-les-règles-du-logiciel-de-vol)
13. [Les tests](#13-les-tests)
14. [Commandes et outils](#14-commandes-et-outils)
15. [Les décisions prises](#15-les-décisions-prises)
16. [Aide-mémoire Rust pour qui vient de JavaScript](#16-aide-mémoire-rust-pour-qui-vient-de-javascript)
17. [Pièges déjà rencontrés](#17-pièges-déjà-rencontrés)
18. [Glossaire](#18-glossaire)
19. [Écarts avec un vrai satellite](#19-écarts-avec-un-vrai-satellite)
20. [Méthode de travail](#20-méthode-de-travail)
21. [Tenir ce guide à jour](#21-tenir-ce-guide-à-jour)
22. [Références](#22-références)

## 1. Le projet en une page

Sagittarius est une mission de satellite **simulée**. Le satellite, Sagittarius-1, n'existe pas : c'est un programme qui joue son rôle, face à un autre programme qui joue l'espace autour de lui.

Le but est d'apprendre le logiciel de vol embarqué tel qu'il se pratique dans l'industrie, en l'écrivant. L'auteur vient du développement web et débute en Rust et dans le spatial.

**Le satellite.** Un CubeSat 6U : environ 10 × 20 × 30 cm, la taille d'une boîte à chaussures, pour 12 kg au plus. Il a une propulsion chimique à monergol vert, le LMP-103S.

**La mission.** Le lanceur le largue à environ 400 km d'altitude. Il doit monter seul jusqu'à 550 km :

1. Séparation du lanceur : le logiciel de vol démarre en mode SAFE.
2. Réduction de la rotation (detumbling).
3. Déploiement des panneaux solaires.
4. Premier contact radio avec la station sol.
5. Recette des sous-systèmes, passage en mode NOMINAL.
6. Montée d'orbite par transfert de Hohmann : deux allumages, environ 85 m/s au total.
7. Passage en mode MISSION.

Les étapes 1 à 5 forment la LEOP, la phase la plus risquée d'une mission. Les mots en capitales sont dans le [glossaire](#18-glossaire).

**Ce qu'on construit.** Trois choses :

- le **logiciel de vol**, en Rust : le programme qui tournerait dans l'ordinateur de bord ;
- le **simulateur**, en Rust : l'orbite, la batterie, la température, les équipements, la radio ;
- le **segment sol**, en TypeScript : une passerelle et une interface web pour suivre et commander le satellite.

**L'idée centrale.** Le logiciel de vol ne sait pas qu'il est simulé. Il est écrit comme pour une vraie carte : sans allocation dynamique, sans système d'exploitation supposé, en interrogeant ses équipements sur un bus. Tout ce qui dépend de la machine est caché derrière quelques interfaces, les traits de plateforme. En phase 5, il suffira de réécrire ces interfaces pour le faire tourner sur une carte STM32.

**Ce que le projet n'est pas.** Il n'utilise ni cFS, le framework de la NASA, ni PUS, le protocole standard européen. Le framework et le protocole sont faits maison, inspirés des deux, pour comprendre ce qu'ils font.

## 2. Reprendre en dix minutes

1. **Voir où on est.**

   ```bash
   git status
   ```

   ```bash
   git log --oneline -10
   ```

2. **Lire [`en-cours.md`](en-cours.md) s'il existe.** C'est la note de l'étape en cours : ce qui reste à écrire, les signatures attendues, les résultats à obtenir. Elle est plus récente et plus détaillée que ce guide.
3. **Lancer les vérifications**, depuis la racine du dépôt, et comparer avec la section [3](#3-où-en-est-le-projet).

   ```bash
   cargo test --workspace
   ```

   ```bash
   cargo lint
   ```

4. **Relire la section [7](#7-suivre-une-température-de-bout-en-bout)** pour se remettre le fonctionnement en tête.

### Installer l'environnement sur une nouvelle machine

- **Rust** : installer `rustup`. La version du compilateur est épinglée dans `rust-toolchain.toml` : elle s'installe toute seule à la première commande `cargo`.
- **Un compilateur C** : le paquet `build-essential`. Rust s'en sert pour l'édition de liens.
- **Node 26**, par `nvm` : `nvm use` lit la version dans `.nvmrc`. Node ne sert qu'au segment sol.

Si `cargo` est introuvable dans un terminal :

```bash
source "$HOME/.cargo/env"
```

## 3. Où en est le projet

**Phase 1 : squelette de bout en bout.** État au 2026-11-26.

### Ce qui existe vraiment

| Programme | État |
| --------- | ---- |
| Protocole (`sgr-protocol`) | Les quatre trames du bus et les formats du PCDU sont écrits et testés. |
| Logiciel de vol (`sgr-flight-core`, `sgr-flight`) | Trois traits de plateforme, le pilote du PCDU, un thermostat et une fonction qui joue un cycle. Testé face à une fausse plateforme. |
| Binaire du logiciel de vol (`sgr-flight-linux`) | Coquille vide : `fn main() {}`. |
| Simulateur (`sgr-sim`) | Le monde et le modèle du PCDU sont écrits. Il manque `Sim`, qui les assemble. Le binaire est une coquille vide. |
| Tests de bout en bout (`sgr-e2e`) | Coquille vide. |
| Passerelle (`ground/gateway`) | Coquille : affiche un message et s'arrête. |
| Interface (`ground/web`) | Coquille : une page avec un titre. |

Rien ne passe encore par le réseau. Le logiciel de vol et le simulateur ne se sont pas encore parlé : chacun est testé seul.

### Les étapes de la phase 1

| Étape | Contenu | État |
| ----- | ------- | ---- |
| Trames du bus | `Tick`, `BusRequest`, `BusResponse`, `CycleDone` | Fait |
| 4 et 5 | Formats du PCDU : `PcduHk`, `PcduRequest` | Fait |
| 6 | Traits `Clock` et `EquipmentBus` | Fait |
| 7 | Pilote du PCDU : `read_hk`, `set_switches` | Fait |
| 8 | Thermostat du réchauffeur de la batterie | Fait |
| 9 | Trait `CycleSync`, fonction `run_cycle` | Fait |
| 10a | Simulateur : le monde (`World`) | Fait |
| 10b | Simulateur : le modèle du PCDU (`Pcdu`) | Fait, une remarque de Clippy à corriger |
| 10c | Simulateur : l'assemblage (`Sim`) | **À écrire** |
| 11 | Premier test de bout en bout dans `sgr-e2e` | À venir |

Après l'étape 11, il restera pour finir la phase 1 : la plateforme Linux en UDP, le binaire du simulateur, l'ordonnanceur et les services du cœur, la liaison radio, la passerelle et un premier affichage dans l'interface.

### Ce que donnent les commandes aujourd'hui

| Commande | Résultat | Pourquoi |
| -------- | -------- | -------- |
| `cargo test -p sgr-protocol` | 46 tests passent | |
| `cargo test -p sgr-flight-core` | 9 tests passent | |
| `cargo test -p sgr-flight` | 31 tests passent | |
| `cargo test -p sgr-sim --test world --test pcdu_model` | 22 tests passent | |
| `cargo test -p sgr-sim --test sim` | **ne compile pas** | `Sim` n'est pas écrit (étape 10c). Les 13 tests l'attendent. |
| `cargo test --workspace` | **ne compile pas** | Même raison. |
| `cargo lint` | **échoue** | Même raison, plus `let_and_return` dans le modèle du PCDU. |
| `cargo fmt --all --check` | passe | |
| `cargo check-nostd` | passe | |
| `npm run typecheck` et `npm test` dans `ground/gateway` | passent, 0 test | |
| `npm run lint` et `npm run build` dans `ground/web` | passent | |

Le rouge est voulu : les tests d'une étape sont écrits avant son code. Il disparaît avec l'étape 10c.

### La feuille de route

| Phase | Contenu |
| ----- | ------- |
| 0 | Mise en place : dépôt, workspace, projets Node et React, ICD v0. **Faite.** |
| 1 | Squelette de bout en bout : simulateur et logiciel de vol en lockstep, liaison radio, quelques valeurs affichées. **En cours.** |
| 2 | TC/TM et sûreté : protocole complet, acquittements, paramètres, modes, FDIR, injection de pannes. |
| 3 | Propulsion : modèle physique, machine à états, manœuvres, scénario LEOP complet. |
| 4 | Attitude : contrôle simplifié, detumbling réel. |
| 5 | Matériel : portage sur STM32 Nucleo, simulateur relié en liaison série. |

Le détail est dans [`conception.md`](conception.md), section « Feuille de route ».

## 4. Vue d'ensemble : quatre programmes

```
ground/web (React)                     l'écran de l'opérateur
     │  WebSocket
ground/gateway (Node + TypeScript)     la station sol
     │  UDP : liaison radio (TM/TC)
     │  UDP, autre port : canal « banc de test »
sgr-sim (Rust)                         le monde : orbite, énergie, thermique, radio
     │  UDP en lockstep : bus d'équipements
sgr-flight-linux (Rust)                le satellite : logiciel de vol
```

| Programme | Rôle | Un équivalent dans le web |
| --------- | ---- | ------------------------- |
| `sgr-flight-linux` | Le logiciel de vol. Une boucle à 10 Hz qui lit ses équipements, décide et commande. | Un serveur sans requêtes : une boucle `setInterval` de 100 ms qui ne doit jamais planter. |
| `sgr-sim` | Tout ce qui entoure le logiciel de vol : la physique, les équipements, le canal radio. | Un faux backend très complet, qui simule aussi le temps. |
| `ground/gateway` | Traduit entre le monde du satellite (UDP, octets) et celui du navigateur (WebSocket). | Un BFF, ou un proxy. |
| `ground/web` | Affiche la télémesure et envoie des télécommandes. | Un tableau de bord. |

Trois canaux circulent entre eux, à ne pas confondre :

- **Le bus d'équipements**, entre le logiciel de vol et le simulateur. Le logiciel de vol y interroge ses équipements. C'est le seul qui existe dans le code aujourd'hui.
- **La liaison radio**, entre le sol et le satellite. Elle passe par le simulateur, qui joue le canal : il coupe la liaison quand le satellite n'est pas au-dessus de la station.
- **Le canal « banc de test »**, entre l'interface et le simulateur. Il sert à injecter des pannes. Il ne passe jamais par la radio : dans la réalité, c'est un câble de laboratoire, pas une fonction du satellite.

## 5. Le cœur du projet : le cycle en lockstep

### Pourquoi

Le logiciel de vol travaille par cycles de 100 ms. Si le simulateur et le logiciel de vol tournaient chacun à son rythme, sur l'horloge du PC, deux exécutions ne donneraient jamais le même résultat : tout dépendrait de la charge de la machine.

Le lockstep règle le problème : **le temps n'avance que quand les deux sont d'accord**. C'est le fonctionnement d'un jeu au tour par tour. Le simulateur donne le numéro du tour, le tick. Le logiciel de vol joue son tour. Le monde avance d'un pas.

Trois conséquences :

- les tests sont reproductibles : deux exécutions donnent exactement le même résultat ;
- ils peuvent tourner bien plus vite que le temps réel, puisque personne n'attend 100 ms ;
- on peut poser un point d'arrêt dans le débogueur sans rien désynchroniser.

Le temps bord vient donc toujours du tick, **jamais de l'horloge de la machine**. Le tick 0 est la séparation du lanceur, et le temps bord vaut `tick` × 100 ms.

### Le déroulement

1. Le simulateur envoie `Tick` avec le tick N.
2. Le logiciel de vol fait son cycle. Chaque fois qu'il a besoin d'un équipement, il envoie une `BusRequest` et attend la `BusResponse`.
3. Le logiciel de vol envoie `CycleDone` avec le tick N.
4. Le simulateur avance le monde de 100 ms et passe au tick N + 1.

```
 simulateur                                  logiciel de vol
     │ ── Tick (tick 1) ──────────────────────────▶ │
     │ ◀── BusRequest (PCDU, GET_HK) ────────────── │
     │ ── BusResponse (OK, 28 V, −5,25 °C) ───────▶ │   le thermostat décide
     │ ◀── BusRequest (PCDU, SET_SWITCHES 0x01) ─── │
     │ ── BusResponse (OK, vide) ─────────────────▶ │
     │ ◀── CycleDone (tick 1) ───────────────────── │
     │  le monde avance de 100 ms
     │ ── Tick (tick 2) ──────────────────────────▶ │
```

Trois règles à retenir :

- **Le logiciel de vol est le maître du bus.** Un équipement ne parle que si on l'interroge.
- **Le monde est figé pendant le cycle.** Toutes les mesures lues pendant le cycle N datent du même instant. Une commande change l'état de l'équipement tout de suite, et ses effets physiques arrivent à l'étape 4.
- **Le silence est dit.** Un équipement qui ne répond pas donne une `BusResponse` de statut `NO_RESPONSE`. Sur un vrai bus, on attendrait un délai. Ici, attendre casserait le temps simulé.

Le format exact des quatre trames est dans [`icd.md`](icd.md), section « Interface lockstep ». Le choix de ce dialogue est expliqué dans la [décision 0001](decisions/0001-dialogue-par-bus-d-equipements.md).

## 6. Lire des octets

Les messages voyagent sous forme d'octets. Cette section donne ce qu'il faut pour lire une trame de l'ICD.

**Un octet** est un nombre de 0 à 255. On l'écrit en hexadécimal, sur deux chiffres : `00` à `FF`. `0x10` vaut 16, `0xFF` vaut 255.

**Les types.** Rust distingue les entiers par leur taille et leur signe, là où JavaScript n'a que `number` :

| Type | Taille | Valeurs |
| ---- | ------ | ------- |
| `u8` | 1 octet | 0 à 255 |
| `u16` | 2 octets | 0 à 65 535 |
| `i16` | 2 octets | −32 768 à +32 767 |
| `u32` | 4 octets | 0 à environ 4,3 milliards |

**Gros-boutiste (big-endian).** Un nombre de plusieurs octets s'écrit avec l'octet de poids fort en premier, comme on écrit les chiffres d'un nombre décimal. `6D 60` vaut `0x6D60`, soit 28 000.

**Les nombres négatifs.** Un `i16` est codé en complément à deux : au-delà de 32 767, on retranche 65 536. `FD F3` vaut 65 011 comme `u16`, donc −525 comme `i16`.

**Pas de virgule sur le fil.** Une grandeur physique voyage comme un entier, avec une échelle fixe. La tension est en millivolts, la température en centièmes de degré : −525 veut dire −5,25 °C.

**Un bit par interrupteur.** L'octet `switches` range huit interrupteurs. Le bit 0, celui de poids faible, est le réchauffeur de la batterie. Pour le lire, on masque : `switches & 0x01`. Le résultat est différent de 0 si le bit est à 1.

### Une trame décodée

La troisième trame de référence de l'ICD, une `BusResponse` :

```
02 03 00 00 00 01 00 6D 60 FD F3 00
```

| Octets | Champ | Valeur |
| ------ | ----- | ------ |
| `02` | `version` | 2, la version de l'interface lockstep |
| `03` | `frame_type` | 3, une `BusResponse` |
| `00 00 00 01` | `tick` | 1 |
| `00` | `status` | 0, `OK` |
| `6D 60` | `bat_v` | 28 000 mV, soit 28 V |
| `FD F3` | `bat_temp` | −525, soit −5,25 °C |
| `00` | `switches` | tous les interrupteurs ouverts |

Les sept premiers octets appartiennent au bus. Les cinq derniers sont la réponse du PCDU : le bus les transporte sans les lire.

En JavaScript, la même lecture se ferait avec un `DataView` :

```js
const view = new DataView(bytes.buffer);
const batV = view.getUint16(7, false);    // 28000, false = big-endian
const batTemp = view.getInt16(9, false);  // -525
```

En Rust, dans [`sgr-protocol`](../crates/sgr-protocol/src/equipment/pcdu.rs) :

```rust
bat_v: u16::from_be_bytes([bytes_decoded[0], bytes_decoded[1]]),
bat_temp: i16::from_be_bytes([bytes_decoded[2], bytes_decoded[3]]),
```

## 7. Suivre une température de bout en bout

Le meilleur moyen de comprendre le projet est de suivre une mesure. La batterie est à 28 V et à 4,994 °C, le réchauffeur est éteint. Voici ce qui se passe pendant un cycle.

| # | Qui | Ce qui se passe | Où |
| - | --- | --------------- | -- |
| 1 | Simulateur | Le monde contient `bat_temp = 4.994`, un nombre à virgule, en degrés. | [`world.rs`](../crates/sgr-sim/src/world.rs) |
| 2 | Vol | `run_cycle` attend le tick : `platform.wait_tick()`. | [`cycle.rs`](../crates/sgr-flight/src/cycle.rs) |
| 3 | Vol | Le pilote demande le housekeeping. `read_hk` encode `GET_HK` en un octet, `01`, et appelle `bus.transfer(0x10, …)`. | [`drivers/pcdu.rs`](../crates/sgr-flight/src/drivers/pcdu.rs) |
| 4 | Plateforme | Elle porte la demande jusqu'au simulateur, qui la donne à l'équipement d'adresse `0x10`. | étapes 10c et 11, à écrire |
| 5 | Simulateur | Le modèle du PCDU mesure le monde : `(4.994 * 100.0).round() as i16` donne 499. Il répond `6D 60 01 F3 00`. | [`equipment/pcdu.rs`](../crates/sgr-sim/src/equipment/pcdu.rs) |
| 6 | Vol | `read_hk` vérifie que la réponse fait 5 octets, la décode et renvoie un `PcduHk` : 28 000 mV, 499, interrupteurs ouverts. | [`drivers/pcdu.rs`](../crates/sgr-flight/src/drivers/pcdu.rs) |
| 7 | Vol | Le thermostat reçoit 499. C'est en dessous de 500 : il décide d'allumer. | [`thermal.rs`](../crates/sgr-flight/src/thermal.rs) |
| 8 | Vol | Le pilote envoie l'état voulu des interrupteurs : `set_switches` encode `02 01`. | [`drivers/pcdu.rs`](../crates/sgr-flight/src/drivers/pcdu.rs) |
| 9 | Simulateur | Le modèle du PCDU retient `switches = 0x01` et répond par une réponse vide. | [`equipment/pcdu.rs`](../crates/sgr-sim/src/equipment/pcdu.rs) |
| 10 | Vol | `run_cycle` signale la fin du cycle : `platform.cycle_done()`. | [`cycle.rs`](../crates/sgr-flight/src/cycle.rs) |
| 11 | Simulateur | Le monde avance de 100 ms, réchauffeur allumé : la batterie gagne 0,00075 °C. Le tick passe au suivant. | [`world.rs`](../crates/sgr-sim/src/world.rs) |

C'est une **boucle fermée** : le logiciel de vol mesure, décide, commande, et le monde réagit. Sur la durée, la température oscille entre 5 et 10 °C. Avec les valeurs actuelles du modèle, le réchauffeur chauffe pendant environ 13 min 30, puis reste éteint pendant environ 9 min 30.

Deux remarques :

- Chaque couche ne connaît que sa voisine. Le thermostat ne voit qu'un nombre. Le pilote ne sait pas ce qu'on fait de la mesure. Le monde ne sait pas qu'il existe un bus.
- Le maillon n° 4 n'existe pas encore. Aujourd'hui, les tests du logiciel de vol le remplacent par une fausse plateforme, et ceux du simulateur appellent le modèle directement. L'étape 11 les branchera l'un sur l'autre, en mémoire. Le réseau viendra ensuite.

## 8. Le dépôt, dossier par dossier

```
CLAUDE.md                  règles et conventions du projet
Cargo.toml                 workspace Rust : liste des crates, dépendances, règles Clippy
rust-toolchain.toml        version épinglée du compilateur
.cargo/config.toml         alias `cargo lint` et `cargo check-nostd`
clippy.toml                les tests ont le droit d'utiliser unwrap, expect et panic
.github/workflows/ci.yml   intégration continue
docs/
  guide.md                 ce guide
  conception.md            conception détaillée et feuille de route
  icd.md                   formats des messages
  decisions/               décisions d'architecture
  en-cours.md              note temporaire de l'étape en cours
crates/
  sgr-protocol/            formats des messages
  sgr-flight-core/         traits de plateforme, plus tard les services du cœur
  sgr-flight/              pilotes d'équipements et sous-systèmes
  sgr-flight-linux/        plateforme Linux et binaire du logiciel de vol
  sgr-sim/                 simulateur
  sgr-e2e/                 tests de bout en bout
ground/
  gateway/                 passerelle UDP <-> WebSocket
  web/                     interface React
```

Une **crate** est un paquet Rust, l'équivalent d'un paquet npm. Le **workspace** les regroupe, comme un monorepo.

### Qui dépend de qui

| Crate | `no_std` | A le droit de dépendre de | Raison |
| ----- | -------- | ------------------------- | ------ |
| `sgr-protocol` | oui | `heapless` | Les formats sont partagés par le vol et le simulateur. |
| `sgr-flight-core` | oui | `sgr-protocol` (pas utilisé aujourd'hui) | Le cœur ne connaît aucun équipement. |
| `sgr-flight` | oui | `sgr-flight-core`, `sgr-protocol` | `sgr-protocol` est réservé aux pilotes. |
| `sgr-flight-linux` | non | `sgr-protocol`, `sgr-flight-core`, `sgr-flight` | La seule crate de vol qui utilise la bibliothèque standard. |
| `sgr-sim` | non | `sgr-protocol` | **Jamais les crates de vol** : le simulateur ne connaît du satellite que son bus. |
| `sgr-e2e` | non | toutes | Il assemble le simulateur et le logiciel de vol. |

Ces limites sont rappelées en commentaire dans le `Cargo.toml` de chaque crate.

### Crate par crate

**`sgr-protocol`** : le dictionnaire commun. Deux modules.

- [`lockstep`](../crates/sgr-protocol/src/lockstep.rs) : les quatre trames du bus (`Tick`, `BusRequest`, `BusResponse`, `CycleDone`), chacune avec `encode` et `decode`. Le type `Payload` est un tableau d'au plus 32 octets, sans allocation.
- [`equipment::pcdu`](../crates/sgr-protocol/src/equipment/pcdu.rs) : ce que le PCDU comprend (`PcduRequest`) et ce qu'il répond (`PcduHk`), son adresse (`PCDU_ADDRESS`) et ses interrupteurs (`SWITCH_BAT_HEATER`).

**`sgr-flight-core`** : pour l'instant un seul module, [`platform`](../crates/sgr-flight-core/src/platform.rs), avec les traits `Clock`, `CycleSync` et `EquipmentBus`, et l'erreur `BusError`. Les services du cœur viendront ici.

**`sgr-flight`** : trois modules.

- [`drivers`](../crates/sgr-flight/src/drivers.rs) : l'erreur `DriverError`, et [`drivers::pcdu`](../crates/sgr-flight/src/drivers/pcdu.rs) avec `read_hk` et `set_switches`.
- [`thermal`](../crates/sgr-flight/src/thermal.rs) : le `Thermostat`.
- [`cycle`](../crates/sgr-flight/src/cycle.rs) : `run_cycle` et `CycleReport`.

**`sgr-sim`** : trois modules.

- [`world`](../crates/sgr-sim/src/world.rs) : la physique.
- [`equipment::pcdu`](../crates/sgr-sim/src/equipment/pcdu.rs) : le modèle du PCDU.
- [`sim`](../crates/sgr-sim/src/sim.rs) : l'assemblage, à écrire.

**`sgr-flight-linux`, `sgr-e2e`, `ground/gateway`, `ground/web`** : des coquilles prêtes à être remplies.

## 9. Le logiciel de vol

### Quatre couches

Chacune ne dépend que de celles du dessous.

| # | Couche | Crate | Rôle | Aujourd'hui |
| - | ------ | ----- | ---- | ----------- |
| 4 | Sous-systèmes | `sgr-flight` | Énergie, thermique, propulsion, attitude, modes et FDIR. La logique du satellite. | Un thermostat. |
| 3 | Pilotes d'équipements | `sgr-flight` | Un par équipement. Traduit des octets en mesures. | Le pilote du PCDU. |
| 2 | Services du cœur | `sgr-flight-core` | Ordonnanceur, bus interne, paramètres, événements, TC/TM, stockage. | Rien encore. |
| 1 | Plateforme | traits dans `sgr-flight-core`, code dans `sgr-flight-linux` | Ce que le logiciel demande à la machine. | Trois traits, pas d'implémentation réelle. |

C'est le découpage de cFS, le framework de la NASA : sa couche plateforme s'appelle OSAL et PSP, ses services du cœur cFE, et ses sous-systèmes sont des applications.

### La plateforme

Un **trait** est une interface, comme en TypeScript. Le logiciel de vol ne connaît que ces trois-là :

```rust
pub trait Clock {
    fn tick(&self) -> u32;                 // le numéro du cycle en cours
}

pub trait CycleSync {
    fn wait_tick(&mut self);               // attend le début du cycle
    fn cycle_done(&mut self);              // signale sa fin
}

pub trait EquipmentBus {
    fn transfer(&mut self, address: u8, request: &[u8], response: &mut [u8])
        -> Result<usize, BusError>;        // un échange demande-réponse
}
```

`transfer` copie la réponse au début du buffer fourni et renvoie sa longueur. Deux cas à ne pas confondre :

- `Ok(0)` : l'équipement a obéi et n'a rien à dire. C'est une **réponse vide**.
- `Err(BusError::NoResponse)` : l'équipement est resté muet. C'est un **silence**.

Sur Linux, ces traits seront implémentés avec une socket UDP. Sur la carte, `wait_tick` attendra un timer matériel et `transfer` parlera à un vrai bus, I²C par exemple. Le reste du logiciel ne changera pas. Voir la [décision 0002](decisions/0002-forme-des-traits-de-plateforme.md).

Un quatrième trait, `GroundLink`, est prévu pour la radio. Sa forme est remise en question : voir [Les questions ouvertes](#les-questions-ouvertes).

### Les pilotes

Un pilote est un jeu de fonctions, une par commande de l'ICD. Pour le PCDU :

```rust
pub fn read_hk<B: EquipmentBus>(bus: &mut B) -> Result<PcduHk, DriverError>
pub fn set_switches<B: EquipmentBus>(bus: &mut B, switches: u8) -> Result<(), DriverError>
```

Chaque fonction encode la demande, fait un échange, vérifie la réponse et la décode. Trois règles, de la [décision 0003](decisions/0003-pilotes-d-equipements.md) :

- **Un échange par appel, jamais de nouvel essai.** Réessayer est une décision du FDIR.
- **Le pilote ne fait pas confiance au bus.** Il vérifie la longueur reçue avant de s'en servir.
- **Deux erreurs seulement** : `NoResponse` et `InvalidResponse`, communes à tous les pilotes.

Le PCDU n'est pas la batterie. C'est le boîtier électrique auquel elle est branchée : il la mesure, et il alimente chaque équipement par un interrupteur.

### Les sous-systèmes

Aujourd'hui, un seul morceau du thermique : le `Thermostat` du réchauffeur de la batterie. Il allume en dessous de +5,00 °C et éteint au-dessus de +10,00 °C. Entre les deux, il ne change rien.

Cet écart entre les deux seuils s'appelle une **hystérésis**. Avec un seul seuil, le réchauffeur claquerait sans arrêt dès que la température tourne autour. La batterie est tenue au chaud parce qu'une cellule lithium-ion ne doit pas être chargée en dessous de 0 °C.

La cible est un trait commun à tous les sous-systèmes, décrit dans [`conception.md`](conception.md) :

```rust
pub trait Subsystem {
    fn step(&mut self, ctx: &mut Context);
    fn handle(&mut self, cmd: &Command, ctx: &mut Context) -> Result<CmdStatus, CmdError>;
    fn housekeeping(&self, out: &mut HkWriter);
}
```

Les sous-systèmes ne se parleront jamais directement, seulement par le bus interne.

### Deux bus à ne pas confondre

- Le **bus d'équipements** relie l'ordinateur de bord à ses équipements. Seuls les pilotes y parlent.
- Le **bus interne** reliera les sous-systèmes entre eux, à l'intérieur du logiciel de vol. Il n'existe pas encore.

### Le cycle d'aujourd'hui

Une seule fonction, [`run_cycle`](../crates/sgr-flight/src/cycle.rs), joue tout le cycle :

1. attendre le tick ;
2. lire le housekeeping du PCDU ;
3. donner la température au thermostat, si la lecture a réussi ;
4. envoyer l'état voulu des interrupteurs ;
5. signaler la fin du cycle.

Elle renvoie un `CycleReport`, avec le résultat de la lecture et celui de la commande. Trois règles, de la [décision 0004](decisions/0004-boucle-minimale.md) :

- **Un cycle va toujours jusqu'au bout.** Sans `CycleDone`, le simulateur attendrait pour toujours.
- **Sans mesure valide, le thermostat garde sa dernière commande.** On ne lui invente pas de valeur.
- **L'état des interrupteurs est envoyé à chaque cycle**, même inchangé. Si le PCDU a redémarré, un cycle suffit à le rétablir.

### Le cycle visé

`run_cycle` sera remplacée par un ordonnanceur. Un cycle de 100 ms fera alors, dans cet ordre :

1. acquisition : les pilotes interrogent les équipements ;
2. traitement des télécommandes reçues ;
3. `step()` de chaque sous-système, dans un ordre fixe ;
4. `step()` des modes et du FDIR, toujours en dernier ;
5. commandes aux équipements, housekeeping, événements.

C'est un **exécutif cyclique** : une seule boucle, sans threads. Les cadences lentes passent par des diviseurs : le housekeeping à 1 Hz part tous les 10 cycles.

## 10. Le simulateur

Trois pièces, chacune dans son module. Voir la [décision 0005](decisions/0005-simulateur-minimal.md).

### Le monde

[`World`](../crates/sgr-sim/src/world.rs) contient la physique, en unités physiques et en nombres à virgule : `bat_temp` en degrés Celsius, `bat_v` en volts. Il ne connaît ni le bus ni les octets.

La batterie est un seul **nœud thermique**. C'est une tasse de café dans une pièce froide : elle perd de la chaleur d'autant plus vite qu'elle est plus chaude que la pièce. Le réchauffeur est une résistance qui lui apporte une puissance fixe. À chaque pas de 100 ms :

```
perte     = (bat_temp − ENV_TEMP) / BAT_THERMAL_RESISTANCE
apport    = BAT_HEATER_POWER si le réchauffeur est alimenté, 0 sinon
bat_temp += (apport − perte) × TICK_SECONDS / BAT_HEAT_CAPACITY
```

| Constante | Valeur | Sens |
| --------- | ------ | ---- |
| `TICK_SECONDS` | 0,1 s | Durée d'un tick |
| `ENV_TEMP` | −10 °C | Température de ce qui entoure la batterie |
| `BAT_HEAT_CAPACITY` | 400 J/K | Énergie qu'il faut pour réchauffer la batterie de 1 °C |
| `BAT_THERMAL_RESISTANCE` | 5 K/W | Un écart de 5 °C fait perdre 1 W |
| `BAT_HEATER_POWER` | 6 W | Puissance du réchauffeur |

Deux valeurs en découlent :

- **Sans réchauffeur**, la batterie descend vers −10 °C. L'écart se réduit de 63 % toutes les 2000 s, soit 33 min : c'est la constante de temps, 5 × 400.
- **Avec le réchauffeur**, elle monte vers +20 °C : −10 + 6 × 5.

La tension ne varie pas encore.

### Le modèle du PCDU

[`Pcdu`](../crates/sgr-sim/src/equipment/pcdu.rs) est le côté simulateur de l'équipement. Il garde l'état réel de ses interrupteurs, et répond aux demandes avec une seule méthode :

```rust
pub fn handle(&mut self, world: &World, request: &[u8]) -> Option<Payload>
```

- `GET_HK` : il mesure le monde et traduit au format de l'ICD. Une mesure est arrondie au pas le plus proche, puis saturée aux bornes de son type.
- `SET_SWITCHES` : il remplace l'état de ses interrupteurs et répond par une réponse vide. Les bits réservés sont ignorés.
- Toute autre demande : `None`, un silence.

Le pilote et le modèle sont les deux bouts du même fil. Ils partagent les formats de `sgr-protocol`. Pour qu'une erreur commune ne passe pas inaperçue, chacun est aussi testé contre les octets de l'ICD, écrits à la main dans ses tests.

### L'assemblage

`Sim` réunira le monde et les équipements derrière le bus. C'est l'étape 10c :

| Méthode | Rôle |
| ------- | ---- |
| `new(world)` | Le monde donné, un PCDU neuf, le tick 0 |
| `tick()` | Le tick en cours |
| `bus_request(address, request)` | Donne la demande à l'équipement dont c'est l'adresse, sinon `None` |
| `step()` | Avance le monde de 100 ms, puis passe au tick suivant |

Seul `step` change le monde.

### Ce qui viendra

L'orbite, les éclipses et les fenêtres de visibilité, la production des panneaux et la charge de la batterie, d'autres nœuds thermiques, la propulsion, le canal radio, le bruit de mesure et les pannes. Voir [`conception.md`](conception.md), section « Simulateur ».

## 11. Le segment sol

Les deux projets sont en place, vides.

**La passerelle** (`ground/gateway`) fera le pont entre UDP et WebSocket, encodera et décodera le protocole sol-bord, et archivera la télémesure. Node exécute directement les fichiers `.ts` : il n'y a pas d'étape de build, et `tsc` ne sert qu'à vérifier les types.

**L'interface** (`ground/web`, React et Vite) affichera la carte, les tableaux de bord, le journal d'événements, une console de télécommandes, la planification des manœuvres et le panneau d'injection de pannes.

Le protocole sol-bord sera donc écrit deux fois : en Rust à bord, en TypeScript au sol. Pour détecter une divergence, les deux côtés seront testés contre les mêmes **paquets de référence**, versionnés avec l'ICD.

Le détail est dans [`conception.md`](conception.md), sections « Protocole sol-bord » et « Segment sol ».

## 12. Les règles du logiciel de vol

Un satellite ne se redémarre pas à la main, et personne ne viendra le déboguer. Son logiciel doit donc avoir un comportement prévisible, quoi qu'il arrive. Ces règles viennent de là. Elles reprennent l'esprit des « dix règles » du JPL pour le code critique (Holzmann, *The Power of Ten*).

| Règle | Pourquoi | Qui la fait respecter |
| ----- | -------- | --------------------- |
| Aucune allocation dynamique après l'initialisation | Un tas peut se fragmenter ou s'épuiser après des mois de vol. Avec des tailles fixes, la mémoire nécessaire est connue à la compilation. | `cargo check-nostd` : sans bibliothèque standard, `Vec` et `Box` n'existent pas. On utilise `heapless`. |
| Pas de `unwrap()`, `expect()` ni `panic!` | Une panique arrête le programme. À bord, chaque erreur doit être traitée ou signalée. | `cargo lint` : Clippy les refuse. Les tests en sont dispensés (`clippy.toml`). |
| Trois crates en `no_std` | Sur la carte, il n'y a pas de système d'exploitation, donc pas de bibliothèque standard. | `cargo check-nostd` : compile pour une cible sans système. |
| Pas de code `unsafe` | Dans un bloc `unsafe`, le compilateur ne garantit plus la sûreté de la mémoire. Seule exception prévue : une crate `*-sys` qui envelopperait un driver C. | `unsafe_code = "forbid"` dans chaque crate. |
| Dans `sgr-flight`, seuls les pilotes utilisent `sgr-protocol`, et seulement son module `equipment` | Les sous-systèmes ne doivent pas dépendre d'un format d'octets. | Le test [`architecture.rs`](../crates/sgr-flight/tests/architecture.rs). |
| Les sous-systèmes ne voient pas le format des paquets sol-bord | Changer de protocole ne doit toucher que le service TC/TM. | La même structure de dépendances. |
| Le temps bord vient du tick | Les tests doivent être reproductibles. | Le trait `Clock`, et la relecture. |
| Toute modification d'un message commence par l'ICD | Un format est lu par plusieurs programmes, parfois dans deux langages. Il lui faut une seule source. | Les trames de référence, vérifiées par les tests. |

Le simulateur et les tests de bout en bout tournent au sol : ils ont droit à la bibliothèque standard et aux allocations.

## 13. Les tests

### L'organisation

- **Les tests sont écrits avant le code.** Une étape est finie quand ses tests passent tous.
- **Ils vivent dans le dossier `tests/` de chaque crate.** Un fichier de `tests/` est compilé comme un programme extérieur : il ne voit que ce qui est `pub`. Un `pub` oublié casse donc la compilation. Seule exception : les tests unitaires de `sgr-protocol`, dans `src/…/tests.rs`.
- **Chaque couche se teste seule**, face à un faux de la couche d'en dessous : un pilote face à un faux bus, le cycle face à une fausse plateforme, le modèle du PCDU face à un monde figé.
- **Les tests de bout en bout** (`sgr-e2e`) feront tourner le simulateur et le logiciel de vol dans un seul processus, sans réseau.

### Les fichiers

| Fichier | Tests | Ce qu'il vérifie |
| ------- | ----- | ---------------- |
| `sgr-protocol/src/lockstep/tests.rs` | 32 | Les quatre trames, à l'octet près, et chaque cas de trame invalide |
| `sgr-protocol/src/equipment/pcdu/tests.rs` | 12 | Les formats du PCDU |
| `sgr-protocol/tests/reference_cycle.rs` | 2 | Le cycle de référence de l'ICD, vu de chaque côté |
| `sgr-flight-core/tests/platform.rs` | 9 | La forme des trois traits, en les implémentant avec des faux |
| `sgr-flight/tests/pcdu_driver.rs` | 10 | Le pilote : demandes envoyées, décodage, équipement muet, bus qui ment |
| `sgr-flight/tests/thermal.rs` | 11 | Le thermostat : seuils, hystérésis, valeurs extrêmes |
| `sgr-flight/tests/cycle.rs` | 7 | L'ordre du cycle, et son comportement quand le PCDU est muet |
| `sgr-flight/tests/architecture.rs` | 3 | La règle d'import de `sgr-protocol` |
| `sgr-sim/tests/world.rs` | 12 | Le modèle thermique : un pas, l'équilibre, la constante de temps |
| `sgr-sim/tests/pcdu_model.rs` | 10 | Le modèle du PCDU : trames de l'ICD, arrondi, saturation, silence |
| `sgr-sim/tests/sim.rs` | 13 | L'assemblage : tick, aiguillage, monde figé pendant le cycle. **Ne compile pas encore.** |

Soit 108 tests qui passent, et 121 une fois l'étape 10c écrite.

### Lancer une partie des tests

Tous les tests d'une crate :

```bash
cargo test -p sgr-flight
```

Un seul fichier de `tests/` :

```bash
cargo test -p sgr-flight --test thermal
```

Les tests dont le nom contient un mot :

```bash
cargo test -p sgr-flight heater_turns_on
```

## 14. Commandes et outils

### Rust, depuis la racine

| Commande | Rôle |
| -------- | ---- |
| `cargo build --workspace` | Tout compiler |
| `cargo test --workspace` | Tous les tests |
| `cargo fmt --all` | Formater le code |
| `cargo lint` | Clippy, avec les règles du logiciel de vol. Un avertissement est une erreur. |
| `cargo check-nostd` | Compiler les trois crates `no_std` pour une cible sans système (`thumbv7em-none-eabihf`) |
| `cargo run -p sgr-sim` | Lancer le simulateur (ne fait rien pour l'instant) |
| `cargo run -p sgr-flight-linux` | Lancer le logiciel de vol (ne fait rien pour l'instant) |

`lint` et `check-nostd` sont des alias, définis dans `.cargo/config.toml`.

**Avant de demander une relecture ou de commiter**, quatre commandes doivent passer : `cargo fmt --all`, `cargo test --workspace`, `cargo lint` et `cargo check-nostd`.

### Segment sol

Après `npm install` dans le dossier concerné :

| Dossier | Commandes |
| ------- | --------- |
| `ground/gateway` | `npm run dev`, `npm run typecheck`, `npm test` |
| `ground/web` | `npm run dev`, `npm run lint`, `npm run build` |

### Intégration continue

Le fichier `.github/workflows/ci.yml` rejoue ces vérifications à chaque pull request et à chaque push sur `main` : formatage, Clippy, compilation `no_std` et tests pour Rust, puis les vérifications de la passerelle et de l'interface.

### Versions

- Rust 1.100.0, édition 2024, épinglé dans `rust-toolchain.toml`.
- Node 26, épinglé dans `.nvmrc`.
- `heapless` 0.9 pour les collections à taille fixe.

## 15. Les décisions prises

Une décision d'architecture importante est un court fichier dans [`decisions/`](decisions/). Chacun a la même forme : contexte, décision, conséquences, écarts avec un vrai satellite, ce qui reste ouvert.

| # | Décision | En une phrase |
| - | -------- | ------------- |
| [0001](decisions/0001-dialogue-par-bus-d-equipements.md) | Dialogue par bus d'équipements | Le logiciel de vol interroge ses équipements un par un, au lieu de recevoir toutes les mesures d'un coup. C'est ce qui crée la couche des pilotes. |
| [0002](decisions/0002-forme-des-traits-de-plateforme.md) | Forme des traits de plateforme | `Clock` donne le tick. `EquipmentBus` a une seule méthode bloquante, sur le modèle d'`embedded-hal`. |
| [0003](decisions/0003-pilotes-d-equipements.md) | Forme des pilotes | Un module par équipement, une fonction par commande, le bus prêté le temps de l'appel, une erreur commune. |
| [0004](decisions/0004-boucle-minimale.md) | Boucle minimale | Le trait `CycleSync` et la fonction `run_cycle`, en attendant l'ordonnanceur. Un cycle va toujours jusqu'au bout. |
| [0005](decisions/0005-simulateur-minimal.md) | Simulateur minimal | Trois pièces : le monde, le modèle d'équipement, l'assemblage. Un nœud thermique pour la batterie. |

### Les questions ouvertes

Elles sont à trancher avant d'écrire l'ordonnanceur, chacune par une décision. Les pistes ci-dessous ont été proposées pendant les revues. **Aucune n'est encore validée.**

| Question | Piste proposée |
| -------- | -------------- |
| **La liaison radio.** `CLAUDE.md` et `conception.md` en font un trait de plateforme, `GroundLink`. La décision 0001 et l'ICD penchent pour un équipement émetteur-récepteur sur le bus. | L'équipement sur le bus, pratique courante sur un CubeSat, et un service TC/TM sans entrée-sortie. |
| **Des pilotes aux sous-systèmes.** Comment une mesure parvient-elle aux sous-systèmes ? | Une table de données, que le `Context` donne à lire. |
| **Des sous-systèmes aux équipements.** Comment une commande parvient-elle au pilote ? | Une table d'états voulus, appliquée en fin de cycle avec les inhibitions du FDIR. |
| **Sémantique du bus interne.** Un message publié au cycle N est-il lu au cycle N ou au cycle N + 1 ? | Toujours au cycle suivant : le résultat ne dépend plus de l'ordre d'appel. |
| **Emplacement de `Command`.** Le service TC/TM doit connaître les commandes sans que le cœur dépende de `sgr-flight`. | Des commandes typées, définies dans `sgr-flight` : le cœur devient générique sur le type de message. |

Les trois dernières sont détaillées dans [`conception.md`](conception.md), section « Questions ouvertes ».

## 16. Aide-mémoire Rust pour qui vient de JavaScript

Les notions déjà rencontrées dans le projet, chacune avec son équivalent TypeScript et un exemple tiré du dépôt.

### Valeurs et types

| Rust | TypeScript | Remarque |
| ---- | ---------- | -------- |
| `let x = 5;` | `const x = 5;` | Non modifiable par défaut |
| `let mut x = 5;` | `let x = 5;` | `mut` autorise la modification |
| `u8`, `u16`, `i16`, `u32`, `usize` | `number` | Des entiers de taille fixe. `usize` sert aux tailles et aux indices. |
| `f64` | `number` | Le nombre à virgule. Un littéral s'écrit `0.0`, pas `0`. |
| `bool` | `boolean` | |
| `[u8; 5]` | un tuple de 5 nombres | Tableau de taille fixe, connue à la compilation |
| `&[u8]` | `Uint8Array` en lecture seule | Une vue sur des octets qui appartiennent à quelqu'un d'autre |
| `&mut [u8]` | `Uint8Array` | La même vue, avec le droit d'écrire |
| `x as i16` | `Math.trunc(x)` | Conversion explicite : coupe les décimales et sature aux bornes du type. Rust ne mélange jamais deux types de nombres tout seul. |
| `pub const LEN: usize = 5;` | `export const LEN = 5;` | |

### Structures et méthodes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Thermostat {
    is_heat: bool,
}

impl Thermostat {
    pub fn new() -> Self {
        Self { is_heat: false }
    }

    pub fn heater_on(&self) -> bool {
        self.is_heat
    }

    pub fn update(&mut self, bat_temp: i16) { /* ... */ }
}
```

- `struct` déclare les données, `impl` les méthodes. Ensemble, c'est une classe.
- `Self` est le type lui-même. `new` est un constructeur par convention, pas un mot-clé.
- `&self` : la méthode lit l'objet. `&mut self` : elle le modifie. C'est écrit dans la signature.
- `pub` rend visible hors du module. Sans `pub`, un champ est privé.
- **La dernière expression d'une fonction, sans `;`, est sa valeur de retour.** `self.is_heat` suffit, sans `return`.
- `#[derive(...)]` fait écrire des méthodes par le compilateur : `Debug` pour afficher, `Clone` et `Copy` pour copier, `PartialEq` et `Eq` pour comparer avec `==`, `Default` pour une valeur par défaut.

### Énumérations et `match`

```rust
pub enum PcduRequest {
    GetHk,
    SetSwitches { switches: u8 },
}
```

En TypeScript, c'est une union discriminée :

```ts
type PcduRequest = { kind: "GetHk" } | { kind: "SetSwitches"; switches: number };
```

`match` est un `switch` qui a une valeur, et que le compilateur refuse s'il manque un cas :

```rust
match *self {
    Self::GetHk => Payload::from_array([GET_HK]),
    Self::SetSwitches { switches } => Payload::from_array([SET_SWITCHES, switches]),
}
```

`if` a une valeur lui aussi : `let x = if cond { 1 } else { 0 };` remplace l'opérateur ternaire.

### Absence et erreurs

Rust n'a ni `null` ni exceptions. Les deux cas sont écrits dans le type de retour.

| Rust | TypeScript | Sens |
| ---- | ---------- | ---- |
| `Option<T>` : `Some(x)` ou `None` | `T \| null` | Une valeur, ou rien |
| `Result<T, E>` : `Ok(x)` ou `Err(e)` | une valeur renvoyée, ou une erreur levée | Un succès, ou une erreur |
| `let len = exchange(...)?;` | laisser remonter une exception | En cas d'erreur, la fonction renvoie cette erreur tout de suite |
| `if let Ok(hk) = acquisition { ... }` | `if (r.ok) { const hk = r.value; ... }` | Entrer dans le bloc seulement en cas de succès |

On choisit `Option` quand il n'y a qu'une raison de ne rien renvoyer, et `Result` quand l'appelant a besoin de savoir pourquoi.

### Modules

| Rust | TypeScript |
| ---- | ---------- |
| `pub mod pcdu;` | déclarer qu'un fichier `pcdu.rs` fait partie de la crate |
| `use a::b::{X, Y};` | `import { X, Y } from "a/b";` |
| `use a::b;` puis `b::X` | `import * as b from "a/b";` puis `b.X` |
| `pub use a::b::X;` | `export { X } from "a/b";` |
| `crate::drivers` | un chemin depuis la racine du paquet |

### Traits et génériques

| Rust | TypeScript |
| ---- | ---------- |
| `pub trait Clock { fn tick(&self) -> u32; }` | `interface Clock { tick(): number }` |
| `impl Clock for FakeClock { ... }` | `class FakeClock implements Clock { ... }` |
| `fn read_hk<B: EquipmentBus>(bus: &mut B)` | `function readHk<B extends EquipmentBus>(bus: B)` |
| `P: CycleSync + EquipmentBus` | `P extends CycleSync & EquipmentBus` |

### Emprunts

Chaque valeur a un seul propriétaire. Une fonction peut l'**emprunter** au lieu de la prendre :

- `&x` : un prêt en lecture. Il peut y en avoir plusieurs à la fois.
- `&mut x` : un prêt en écriture. Il est exclusif, et le compilateur le vérifie.

C'est ce qui garantit qu'il n'y a qu'un échange à la fois sur le bus : `transfer` prend `&mut self`.

### Tests et commentaires

```rust
#[test]
fn heater_is_off_at_start() {
    let thermostat = Thermostat::new();
    assert!(!thermostat.heater_on());
}
```

- `#[test]` marque un test. `assert!` vérifie une condition, `assert_eq!` une égalité.
- Un nom suivi de `!` est une macro, pas une fonction : `assert_eq!`, `println!`, `todo!`.
- `//` est un commentaire, `///` documente ce qui suit, `//!` documente le fichier.

### Embarqué

| Rust | Sens |
| ---- | ---- |
| `#![no_std]` | La crate n'utilise pas la bibliothèque standard : ni fichiers, ni réseau, ni allocation. |
| `heapless::Vec<u8, 32>` | Un tableau de longueur variable, mais d'au plus 32 éléments, sans allocation. `push` peut échouer. |
| `x.to_be_bytes()`, `u16::from_be_bytes(...)` | Convertir un entier en octets gros-boutistes, et inversement |
| `1 << 0`, `a & b` | Décalage et masque de bits, comme en JavaScript |

### Pas encore pratiqué

Écrire un `impl Trait for Type` (prévu à l'étape 11, il y en a à lire dans les tests), les durées de vie (`'a`), les fermetures et les itérateurs, `async`.

Pour aller plus loin : *The Rust Programming Language* (« le Book »), et *The Embedded Rust Book* pour le `no_std`.

## 17. Pièges déjà rencontrés

| Symptôme | Cause | Remède |
| -------- | ----- | ------ |
| `cargo` introuvable | Le terminal n'a pas chargé l'environnement de Rust | `source "$HOME/.cargo/env"` |
| `cargo fmt --all --check` échoue | Le code n'a pas été formaté | `cargo fmt --all` |
| `error[E0425]: cannot find type` après un `use a::b;` | On a importé le module, pas ses noms | `use a::b::{X};`, ou écrire `b::X` |
| `error[E0599]: no method named …` dans un test | La méthode attendue par le test n'est pas encore écrite, ou pas `pub` | L'écrire |
| Clippy : `let_and_return` | Une valeur rangée dans une variable, renvoyée à la ligne suivante | Renvoyer l'expression directement |
| Clippy : `new_without_default` | Une struct a un `new()` sans argument et pas de `Default` | Ajouter `Default` au `derive` |
| `derive(Eq)` refusé | La struct contient un `f64` | Garder seulement `PartialEq` |
| Une mesure vaut 28 au lieu de 29 | `0.29 * 100.0` vaut `28.999999999999996`, et `as` coupe les décimales | `.round()` avant `as` |
| Le programme panique sur `&buffer[..length]` | `length` vient de l'extérieur et dépasse le buffer | Vérifier la longueur avant de s'en servir |
| `warning: unused dependency` | Une dépendance est déclarée mais pas encore utilisée | Normal en cours d'étape |
| `node` ou `npm` introuvable | `nvm` n'est pas chargé | `nvm use` dans un terminal interactif |

Et quatre confusions à éviter :

- **Le PCDU n'est pas la batterie**, c'est le boîtier qui la mesure et distribue le courant.
- **Une réponse vide n'est pas un silence** : `Ok(0)` contre `Err(NoResponse)`.
- **Le bus d'équipements n'est pas le bus interne.**
- **Les températures du logiciel de vol sont en centièmes de degré** : 500 vaut +5,00 °C. Celles du monde simulé sont en degrés.

## 18. Glossaire

### Spatial

| Terme | Sens |
| ----- | ---- |
| **ACK** | Acquittement : le satellite confirme qu'il a reçu, accepté ou terminé une télécommande. |
| **ADN** | Dinitramide d'ammonium, le sel à la base du LMP-103S. |
| **Blowdown** | Réservoir pressurisé une fois pour toutes. La pression baisse à mesure que l'ergol est consommé, et la poussée avec elle. |
| **CCSDS** | Le comité qui normalise les protocoles spatiaux. Son *Space Packet* fournit l'en-tête des paquets sol-bord. |
| **cFS** | *core Flight System*, le framework de logiciel de vol de la NASA, en C. |
| **CubeSat** | Format standard de petit satellite. Une unité (1U) est un cube de 10 cm. Un 6U fait environ 10 × 20 × 30 cm. |
| **Detumbling** | Réduction de la rotation du satellite après son éjection. |
| **Δv** (delta-v) | Changement de vitesse qu'une manœuvre apporte, en m/s. C'est la « monnaie » des manœuvres. |
| **Éclipse** | Passage du satellite dans l'ombre de la Terre : plus de soleil sur les panneaux. |
| **FDIR** | *Fault Detection, Isolation and Recovery* : détecter une panne, l'isoler, s'en remettre. Le sous-système qui surveille tous les autres. |
| **Fenêtre de visibilité**, ou **passage** | Les quelques minutes où le satellite est au-dessus de l'horizon de la station sol. Hors de là, pas de radio. |
| **HK**, **housekeeping** | Télémesure de servitude : les mesures de santé du satellite, envoyées régulièrement. |
| **Hohmann** (transfert de) | La façon la plus économe de passer d'une orbite circulaire à une autre : un allumage pour partir, un second, une demi-orbite plus tard, pour arriver. |
| **Isp** | Impulsion spécifique, en secondes : le rendement d'un propulseur. Plus elle est grande, moins on consomme d'ergol pour le même Δv. |
| **LEOP** | *Launch and Early Orbit Phase* : du lancement à la mise en service. |
| **Lit catalytique** | La partie du propulseur où l'ergol se décompose. Il doit être préchauffé avant un allumage. |
| **LMP-103S** | Le monergol vert retenu. Voir `conception.md`, section « Propulsion ». |
| **Mode** | L'état global du satellite : SAFE (survie), NOMINAL, MISSION. |
| **Monergol** | Ergol qui se décompose seul, sans comburant. « Vert » : moins toxique que l'hydrazine. |
| **PCDU** | *Power Conditioning and Distribution Unit* : le boîtier d'alimentation. Il mesure la batterie et alimente chaque équipement par un interrupteur. |
| **PUS** | *Packet Utilization Standard*, la norme européenne des services TC/TM. Le protocole maison s'en inspire. |
| **Segment sol** | Tout ce qui reste sur Terre : station, centre de contrôle, logiciels. |
| **TC** | Télécommande : un ordre du sol vers le satellite. |
| **TM** | Télémesure : les données du satellite vers le sol. |
| **Tsiolkovski** (équation de) | Δv = Isp × g₀ × ln(m₀ / m_f). Elle relie le Δv à la masse d'ergol consommée. |

### Logiciel et embarqué

| Terme | Sens |
| ----- | ---- |
| **ADR** | *Architecture Decision Record* : une décision d'architecture, écrite en une page. |
| **Bus d'équipements** | Le fil sur lequel l'ordinateur de bord interroge ses équipements. |
| **Bus interne** | Le système de messages entre sous-systèmes, à l'intérieur du logiciel de vol. |
| **CRC** | Code de contrôle ajouté à un message pour détecter une erreur de transmission. |
| **`embedded-hal`** | Le jeu de traits standard de Rust pour parler au matériel. |
| **Exécutif cyclique** | Une boucle unique qui refait les mêmes tâches à cadence fixe, sans threads. |
| **Gros-boutiste** (big-endian) | Octet de poids fort en premier. |
| **`heapless`** | Crate de collections à capacité fixe, sans allocation. |
| **HIL** | *Hardware-in-the-loop* : le vrai calculateur branché à un simulateur. |
| **Hystérésis** | Deux seuils au lieu d'un, pour éviter de basculer sans arrêt. |
| **I²C, SPI, CAN** | Des bus réels, qu'on trouve sur une carte. |
| **ICD** | *Interface Control Document* : le document qui fixe les formats des messages. |
| **Lockstep** | Le simulateur et le logiciel de vol avancent ensemble, un pas à la fois. |
| **`no_std`** | Du Rust sans bibliothèque standard, pour une machine sans système d'exploitation. |
| **Payload** | Le contenu utile d'une trame, que le transport ne lit pas. |
| **Pilote** | Le code qui dialogue avec un équipement et traduit ses octets. |
| **Plateforme** | Ce qui dépend de la machine : Linux aujourd'hui, STM32 plus tard. |
| **Saturation** | Une valeur trop grande est ramenée à la borne du type, au lieu de reboucler. |
| **Sous-système** | Un domaine fonctionnel du satellite : énergie, thermique, propulsion… |
| **STM32 Nucleo** | La carte de développement visée en phase 5. |
| **Tick** | Le numéro du cycle. Un tick dure 100 ms. |
| **Trait** | Une interface, en Rust. |
| **Trame** | Un message complet sur un lien, avec son en-tête. |

## 19. Écarts avec un vrai satellite

Le projet simplifie, et le dit. Chaque décision et l'ICD ont une section sur ce point. Les principaux écarts :

- **Un seul bus, générique.** Un vrai satellite en a plusieurs, de types différents.
- **Le silence est explicite**, et le monde est figé pendant le cycle. À bord, chaque échange prend du temps, et un équipement muet se constate par un délai dépassé.
- **Des unités physiques sur le fil.** Un vrai capteur livre une valeur brute, à convertir avec une courbe d'étalonnage.
- **Le temps bord est un compteur de cycles.** Un vrai satellite date en secondes depuis une époque, et se recale sur le sol ou le GPS.
- **Un pilote sans mémoire.** Un vrai pilote compte ses erreurs et garde la date de sa dernière mesure valide.
- **Des seuils en dur** dans le thermostat. Ils seraient des paramètres modifiables depuis le sol.
- **Un seul nœud thermique**, avec une loi linéaire. Un vrai modèle en compte des dizaines, et rayonne en T⁴.
- **Ni cFS ni PUS**, et pas de base de données TM/TC partagée entre le bord et le sol.
- **Une seule boucle, sans système temps réel.**

## 20. Méthode de travail

Le projet est un projet d'apprentissage : l'auteur écrit le logiciel de vol et le simulateur, l'assistant explique, relit et génère le boilerplate.

- **Une notion Rust à la fois**, expliquée depuis zéro et rapprochée de TypeScript.
- **Le guidage est dégressif.** Serré jusqu'au premier test de bout en bout, il se desserre à chaque étape. Ensuite, l'auteur conçoit et écrit seul, tests compris.
- **Une étape qui engage la conception commence par une proposition discutée**, puis par une courte décision dans `decisions/`.
- **Les tests de chaque étape sont préparés avant le code.** Ils sont validés hors du dépôt, contre une implémentation jetable et contre des versions cassées exprès. Seuls les tests entrent dans le dépôt.
- **Chaque étape finit sur un résultat exact** : `N passed; 0 failed`, que l'auteur vérifie avant de demander la relecture.
- **La relecture s'appuie sur les outils** : tests, `cargo lint`, `cargo fmt`, `cargo check-nostd`. Chaque remarque est rattachée au message du compilateur.
- **Les commentaires de documentation** (`///`) sont écrits par l'assistant après le code, puis relus par l'auteur.
- **L'auteur commite lui-même.**

### Conventions

- Le code, les identifiants et les commentaires sont en anglais. La documentation est en français.
- Les noms de l'ICD sont repris tels quels dans le code : `BAT_TEMP` devient `bat_temp`.
- Une modification de message commence par l'ICD, puis passe dans `sgr-protocol`.

### Travailler sur deux machines

Le travail en cours est noté dans [`en-cours.md`](en-cours.md), poussé avec la branche. Avant de fusionner la branche, ce fichier est supprimé, avec la ligne de `CLAUDE.md` qui y renvoie.

## 21. Tenir ce guide à jour

| Quand | Quoi mettre à jour |
| ----- | ------------------ |
| À la fin d'une étape | La section 3, et le nombre de tests de la section 13 |
| À chaque nouvelle décision | Le tableau de la section 15, et les questions ouvertes |
| À chaque nouvelle crate, équipement ou trait | Les sections 8, 9 ou 10 |
| À chaque nouvelle notion Rust acquise | La section 16 |
| À chaque piège qui a coûté du temps | La section 17 |
| À la fin d'une phase | La date en tête du guide, et une relecture complète |

## 22. Références

La liste complète est à la fin de [`conception.md`](conception.md). Les plus utiles pour reprendre :

- **Logiciel de vol** : NASA cFS, pour l'architecture en couches. G. Holzmann, *The Power of Ten: Rules for Developing Safety-Critical Code* (JPL, 2006), pour les règles de codage.
- **Gestion des pannes** : *NASA Fault Management Handbook* (NASA-HDBK-1002).
- **Mécanique orbitale** : H. Curtis, *Orbital Mechanics for Engineering Students*, pour le transfert de Hohmann et l'équation de Tsiolkovski.
- **Conception d'une mission** : Wertz, Everett et Puschell, *Space Mission Engineering: The New SMAD*.
- **Thermique** : D. Gilmore, *Spacecraft Thermal Control Handbook*.
- **Protocoles** : CCSDS 133.0-B (Space Packet Protocol), ECSS-E-ST-70-41C (PUS).
- **CubeSat** : *CubeSat Design Specification*, rév. 14.1 (Cal Poly).
- **Rust** : *The Rust Programming Language*, *The Embedded Rust Book*, et la documentation d'`embedded-hal`.
