# Travail en cours : les traits de plateforme sont écrits, le pilote du PCDU vient ensuite

> **Note temporaire.** Elle sert à reprendre le travail sur une autre machine. Avant de fusionner la branche : supprimer ce fichier et la ligne de `CLAUDE.md` qui y renvoie (section « Avancement »).

## Où on en est

Branche `claude/icd-v0-lockstep`. Phase 1. Le protocole du bus est terminé dans `sgr-protocol`, et le logiciel de vol a ses deux premiers traits de plateforme.

| Fait | Où |
| ---- | -- |
| ICD v0.2 : dialogue par bus d'équipements, premier équipement le PCDU | `docs/icd.md` |
| Décision 0001 : pourquoi le dialogue par bus | `docs/decisions/0001-dialogue-par-bus-d-equipements.md` |
| Les quatre trames du bus, 32 tests | `crates/sgr-protocol/src/lockstep.rs` |
| Étapes 4 et 5 : `PcduHk` et `PcduRequest`, 12 tests | `crates/sgr-protocol/src/equipment/pcdu.rs` |
| Le cycle de référence de l'ICD rejoué par l'API publique, 2 tests | `crates/sgr-protocol/tests/reference_cycle.rs` |
| Décision 0002 : la forme des traits `Clock` et `EquipmentBus` | `docs/decisions/0002-forme-des-traits-de-plateforme.md` |
| Étape 6 : les traits `Clock` et `EquipmentBus`, l'erreur `BusError` | `crates/sgr-flight-core/src/platform.rs` |
| Les traits implémentés par des doublures et utilisés de l'extérieur, 7 tests | `crates/sgr-flight-core/tests/platform.rs` |

Tout est au vert : `cargo test --workspace`, `cargo fmt --all -- --check`, `cargo lint` et `cargo check-nostd`.

## Reprendre sur une autre machine

```bash
git fetch
git switch claude/icd-v0-lockstep
cargo test --workspace
```

Résultat attendu, parmi des lignes à `0 passed` pour les crates encore vides :

```
test result: ok. 7 passed; 0 failed      <- sgr-flight-core, tests/platform.rs
test result: ok. 44 passed; 0 failed     <- sgr-protocol, tests unitaires
test result: ok. 2 passed; 0 failed      <- sgr-protocol, tests/reference_cycle.rs
```

Prérequis : `rustup` (la toolchain épinglée s'installe toute seule) et un compilateur C (`build-essential`). Node n'est pas nécessaire pour l'instant. Si `cargo` est introuvable dans un terminal, lancer `source "$HOME/.cargo/env"`.

## Ensuite

1. **Lever une contradiction** : `crates/sgr-flight/Cargo.toml` interdit de dépendre de `sgr-protocol`, alors que la décision 0001 y range les formats des équipements. À trancher avant le pilote. Proposition : autoriser la dépendance pour les pilotes seulement, qui n'utilisent que `sgr_protocol::equipment`.
2. **Le pilote du PCDU** dans `sgr-flight`. Rien n'est préparé : sa forme est à discuter d'abord (ce qu'il renvoie, comment il signale un équipement muet ou une réponse illisible). Il se sert d'`EquipmentBus`, de `PcduRequest` et de `PcduHk`, et se teste avec un faux bus comme ceux de `crates/sgr-flight-core/tests/platform.rs`.
3. **La cadence du cycle** (attendre le `Tick`, envoyer le `CycleDone`) : un troisième trait de plateforme, puis la boucle qui lit la température et commande le réchauffeur.
4. Le modèle du PCDU dans `sgr-sim`, puis le premier test de bout en bout dans `sgr-e2e`.

## Rappels

- **Le PCDU n'est pas la batterie.** C'est le boîtier électrique auquel elle est branchée : il la mesure, et il alimente chaque équipement par un interrupteur.
- **Un échange avec le PCDU** : le pilote envoie un `PcduRequest` encodé. `GET_HK` reçoit un `PcduHk`, `SET_SWITCHES` une réponse vide.
- **Une réponse vide n'est pas un silence** : `transfer` renvoie `Ok(0)` dans le premier cas, `Err(BusError::NoResponse)` dans le second.
- **Bus d'équipements et bus interne** sont deux choses. Le premier relie l'ordinateur de bord à ses équipements : seuls les pilotes y parlent. Le second relie les sous-systèmes entre eux, à l'intérieur du logiciel de vol.

## Notions Rust déjà vues

Pour doser les explications : celles-ci sont acquises, les autres sont à présenter depuis zéro.

- `struct`, `impl`, constantes associées (`Self::LEN`), `derive`.
- `Result` et `Option`, et comment choisir entre les deux ; l'opérateur `?`.
- Les modules et `use` ; la visibilité (`pub` sur un type, sur chaque champ, sur une constante associée).
- Les `enum` simples et les `enum` qui portent des données ; `match`, y compris sur une tranche d'octets (`[CODE, value]`).
- Les tableaux de taille fixe, `try_into`, `to_be_bytes` et `from_be_bytes`.
- `heapless::Vec` à travers l'alias `Payload` : `from_array`, `from_slice`.
- Déclarer un trait. L'auteur a lu des `impl Trait for Type` et des fonctions génériques (`<B: EquipmentBus>`) dans les tests, mais n'en a pas encore écrit : ce sera la notion nouvelle du pilote.

## Méthode de travail suivie jusqu'ici

Pour une session Claude qui reprendrait sans l'historique de la conversation :

- L'auteur du projet débute en Rust (voir `CLAUDE.md`) : une notion à la fois, expliquée depuis zéro et rapprochée de TypeScript ou de JavaScript.
- Une étape qui engage la conception commence par une proposition discutée, puis par une courte décision dans `docs/decisions/`.
- Claude prépare les tests de chaque étape **avant** le code. Il les valide hors du dépôt contre une implémentation jetable et contre quelques versions cassées exprès, puis ne laisse dans le dépôt que les tests.
- Tout nouveau type public est utilisé par un test du dossier `tests/` de sa crate, pour qu'un `pub` oublié casse la compilation.
- L'auteur écrit le code. Chaque étape se termine sur un résultat exact à obtenir (`N passed; 0 failed`), qu'il vérifie lui-même avant de demander la relecture.
- Claude relit en lançant les tests, `cargo lint`, `cargo fmt` et `cargo check-nostd`, et rattache chaque remarque au message du compilateur correspondant.
- Pour une notion vraiment nouvelle, un exemple complet d'abord, puis l'auteur écrit le cas jumeau seul.
- Les commentaires de documentation (`///`) : l'auteur préfère que Claude les écrive, puis il les relit.
- L'auteur commite lui-même.
