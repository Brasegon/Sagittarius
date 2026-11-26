# Travail en cours : étape 10, le PCDU côté simulateur

> **Note temporaire.** Elle sert à reprendre le travail sur une autre machine. Avant de fusionner la branche : supprimer ce fichier et la ligne de `CLAUDE.md` qui y renvoie (section « Avancement »).

## Où on en est

Branche `claude/icd-v0-lockstep`. Phase 1. Le protocole du bus est terminé dans `sgr-protocol`. Le logiciel de vol sait jouer un cycle complet face à une fausse plateforme. Il lui manque le simulateur en face, puis le premier test de bout en bout.

| Fait | Où |
| ---- | -- |
| ICD v0.2 : dialogue par bus d'équipements, premier équipement le PCDU | `docs/icd.md` |
| Décision 0001 : pourquoi le dialogue par bus | `docs/decisions/0001-dialogue-par-bus-d-equipements.md` |
| Les quatre trames du bus, 32 tests | `crates/sgr-protocol/src/lockstep.rs` |
| Étapes 4 et 5 : `PcduHk` et `PcduRequest`, 12 tests | `crates/sgr-protocol/src/equipment/pcdu.rs` |
| Le cycle de référence de l'ICD rejoué par l'API publique, 2 tests | `crates/sgr-protocol/tests/reference_cycle.rs` |
| Décision 0002 : la forme des traits `Clock` et `EquipmentBus` | `docs/decisions/0002-forme-des-traits-de-plateforme.md` |
| Étape 6 : les traits `Clock` et `EquipmentBus`, l'erreur `BusError` | `crates/sgr-flight-core/src/platform.rs` |
| Décision 0003 : la forme des pilotes d'équipements | `docs/decisions/0003-pilotes-d-equipements.md` |
| Étape 7 : le pilote du PCDU (`read_hk`, `set_switches`) et `DriverError`, 10 tests | `crates/sgr-flight/src/drivers/` et `tests/pcdu_driver.rs` |
| Étape 8 : le thermostat du réchauffeur de la batterie (`Thermostat`), 11 tests | `crates/sgr-flight/src/thermal.rs` et `tests/thermal.rs` |
| Décision 0004 : la boucle minimale | `docs/decisions/0004-boucle-minimale.md` |
| Étape 9 : le trait `CycleSync`, puis `CycleReport` et `run_cycle`, 2 et 7 tests | `crates/sgr-flight-core/src/platform.rs`, `crates/sgr-flight/src/cycle.rs` et leurs tests |
| La règle d'import de `sgr-protocol`, vérifiée par 3 tests | `crates/sgr-flight/tests/architecture.rs` |
| La mission précisée : CubeSat 6U, monergol vert LMP-103S | `docs/conception.md` |

**Tout est au vert** : `cargo test --workspace`, `cargo lint`, `cargo fmt --all --check` et `cargo check-nostd`.

## Reprendre sur une autre machine

```bash
git fetch
git switch claude/icd-v0-lockstep
cargo test --workspace
```

Prérequis : `rustup` (la toolchain épinglée s'installe toute seule) et un compilateur C (`build-essential`). Node n'est pas nécessaire pour l'instant. Si `cargo` est introuvable dans un terminal, lancer `source "$HOME/.cargo/env"`.

## Étape 10 : le PCDU côté simulateur

**La forme est proposée, pas encore décidée.** Elle se discute avec l'auteur avant d'écrire la décision 0005 et les tests.

La proposition tient en trois pièces dans `sgr-sim` :

- **`World`** : l'état physique, en unités physiques et en `f64`. Pour commencer, la température de la batterie, avec un modèle thermique à un seul nœud. `step` l'avance de 100 ms.
- **`Pcdu`** : l'équipement. Il garde l'état de ses interrupteurs et répond aux demandes du bus en mesurant le monde. C'est lui qui convertit une température en centièmes de degré.
- **`Sim`** : il assemble les deux, aiguille une demande vers l'équipement dont c'est l'adresse et compte les ticks.

Les tests du logiciel de vol contiennent deux ébauches du modèle : `PcduModel` dans `tests/pcdu_driver.rs` et `FakePlatform` dans `tests/cycle.rs`.

À trancher avec l'auteur :

- le découpage entre le monde et l'équipement ;
- ce que couvre l'étape 10 : le `Pcdu` seul, ou aussi le modèle thermique ;
- les valeurs du modèle thermique (capacité, fuite vers l'environnement, puissance du réchauffeur).

À faire au passage dans `crates/sgr-sim/Cargo.toml` : ajouter la dépendance à `sgr-protocol`, et corriger le commentaire qui parle encore de l'interface « sensor / actuator » de l'ICD v0.1.

## Ensuite

**Étape 11 : le premier test de bout en bout dans `sgr-e2e`**, le simulateur et le logiciel de vol dans un seul processus. L'auteur y écrira son premier `impl Trait for Type`, pour brancher la plateforme sur le simulateur. C'est la fin de la base : ensuite, il conçoit et écrit seul.

**À décider avant d'écrire l'ordonnanceur**, chacune par une décision :

- **La liaison radio.** `CLAUDE.md` et `docs/conception.md` font de `GroundLink` un trait de plateforme, alors que la décision 0001 et l'ICD penchent pour un équipement émetteur-récepteur sur le bus. Piste proposée par Claude, pas encore validée par l'auteur : l'équipement sur le bus, pratique normale sur un CubeSat, et un service TC/TM sans entrée-sortie, à qui le cycle donne les octets reçus et prend les octets à émettre.
- **Les trois questions ouvertes de `docs/conception.md`**, qui se décident ensemble. Pistes proposées, pas encore validées : une table de données pour les mesures, une table d'états voulus pour les commandes aux équipements, appliquée en fin de cycle avec les inhibitions du FDIR, et un bus interne lu au cycle suivant.

Deux améliorations possibles, laissées à l'auteur :

- `set_switches` donne au bus un buffer de `PcduHk::LEN` octets, alors que la réponse attendue est vide. Un buffer vide (`&mut []`) dirait mieux l'intention. Ce sera nécessaire en I²C, où c'est la taille du buffer qui fixe le nombre d'octets lus.
- Les seuils `500` et `1000` du thermostat sont écrits en dur. Deux constantes nommées diraient ce qu'ils valent et dans quelle unité.

## Rappels

- **Le PCDU n'est pas la batterie.** C'est le boîtier électrique auquel elle est branchée : il la mesure, et il alimente chaque équipement par un interrupteur.
- **Un échange avec le PCDU** : le pilote envoie un `PcduRequest` encodé. `GET_HK` reçoit un `PcduHk`, `SET_SWITCHES` une réponse vide.
- **Une réponse vide n'est pas un silence** : `transfer` renvoie `Ok(0)` dans le premier cas, `Err(BusError::NoResponse)` dans le second.
- **Bus d'équipements et bus interne** sont deux choses. Le premier relie l'ordinateur de bord à ses équipements : seuls les pilotes y parlent. Le second relie les sous-systèmes entre eux, à l'intérieur du logiciel de vol.
- **Seuls les pilotes utilisent `sgr-protocol`** dans `sgr-flight`, et seulement son module `equipment`. Un test le vérifie : `tests/architecture.rs`. Le code au-dessus des pilotes importe ces types depuis le pilote, qui les réexporte.
- **Une longueur qui vient de l'extérieur se vérifie avant de servir** : `&buffer[..length]` panique si `length` dépasse le buffer.
- **Les températures sont en centièmes de degré** : 500 vaut +5,00 °C, 12000 vaut 120 °C.

## Notions Rust déjà vues

Pour doser les explications : celles-ci sont acquises, les autres sont à présenter depuis zéro.

- `struct`, `impl`, constantes associées (`Self::LEN`), `derive`, `let` et `let mut`.
- `Result` et `Option`, et comment choisir entre les deux ; l'opérateur `?`.
- Les modules et `use`, y compris depuis une autre crate ; la visibilité (`pub` sur un type, sur chaque champ, sur une constante associée).
- Les `enum` simples et les `enum` qui portent des données ; `match`, y compris sur une tranche d'octets (`[CODE, value]`) et sur un `Result`.
- Les tableaux de taille fixe, `try_into`, `to_be_bytes` et `from_be_bytes`.
- `heapless::Vec` à travers l'alias `Payload` : `from_array`, `from_slice`.
- Déclarer un trait ; écrire une fonction générique sur un trait (`<B: EquipmentBus>`) ; `todo!()`.
- Une struct qui garde un état, modifié par une méthode `&mut self` ; écrire un test (`#[test]`, `assert!`, `assert_eq!`).
- Deux traits dans une borne (`P: A + B`) ; `pub use` ; `if let`, que l'auteur a trouvé seul.
- Importer un module (`use a::b;`, puis `b::X`) ou ses noms (`use a::b::{X, Y};`), l'équivalent de `import * as b` et de `import { X, Y }`.

Pas encore écrit par l'auteur : un `impl Trait for Type` (il en a lu dans les tests).

## Méthode de travail suivie jusqu'ici

Pour une session Claude qui reprendrait sans l'historique de la conversation :

- L'auteur du projet débute en Rust (voir `CLAUDE.md`) : une notion à la fois, expliquée depuis zéro et rapprochée de TypeScript ou de JavaScript.
- **Le guidage est dégressif.** Il est serré pour la base, jusqu'au premier test de bout en bout, et se desserre d'un cran à chaque étape. Ensuite, l'auteur conçoit et écrit seul, tests compris : Claude relit, explique et génère le boilerplate à la demande.
- Une étape qui engage la conception commence par une proposition discutée, puis par une courte décision dans `docs/decisions/`.
- Claude prépare les tests de chaque étape **avant** le code. Il les valide hors du dépôt contre une implémentation jetable et contre des versions cassées exprès, puis ne laisse dans le dépôt que les tests. Pour le thermostat, l'auteur a écrit une première version des tests, que Claude a complétée à sa demande.
- Tout nouveau type public est utilisé par un test du dossier `tests/` de sa crate, pour qu'un `pub` oublié casse la compilation. L'auteur préfère cet emplacement aux tests en sous-module.
- L'auteur écrit le code. Chaque étape se termine sur un résultat exact à obtenir (`N passed; 0 failed`), qu'il vérifie lui-même avant de demander la relecture.
- Claude relit en lançant les tests, `cargo lint`, `cargo fmt` et `cargo check-nostd`, et rattache chaque remarque au message du compilateur correspondant.
- Les commentaires de documentation (`///`) : l'auteur préfère que Claude les écrive, puis il les relit.
- L'auteur commite lui-même.
