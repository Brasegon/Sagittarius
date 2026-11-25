# Travail en cours : le pilote du PCDU est écrit, la boucle principale vient ensuite

> **Note temporaire.** Elle sert à reprendre le travail sur une autre machine. Avant de fusionner la branche : supprimer ce fichier et la ligne de `CLAUDE.md` qui y renvoie (section « Avancement »).

## Où on en est

Branche `claude/icd-v0-lockstep`. Phase 1. Le protocole du bus est terminé dans `sgr-protocol`, et le logiciel de vol a ses deux premiers traits de plateforme et son premier pilote.

| Fait | Où |
| ---- | -- |
| ICD v0.2 : dialogue par bus d'équipements, premier équipement le PCDU | `docs/icd.md` |
| Décision 0001 : pourquoi le dialogue par bus | `docs/decisions/0001-dialogue-par-bus-d-equipements.md` |
| Les quatre trames du bus, 32 tests | `crates/sgr-protocol/src/lockstep.rs` |
| Étapes 4 et 5 : `PcduHk` et `PcduRequest`, 12 tests | `crates/sgr-protocol/src/equipment/pcdu.rs` |
| Le cycle de référence de l'ICD rejoué par l'API publique, 2 tests | `crates/sgr-protocol/tests/reference_cycle.rs` |
| Décision 0002 : la forme des traits `Clock` et `EquipmentBus` | `docs/decisions/0002-forme-des-traits-de-plateforme.md` |
| Étape 6 : les traits `Clock` et `EquipmentBus`, l'erreur `BusError`, 7 tests | `crates/sgr-flight-core/src/platform.rs` et `tests/platform.rs` |
| Décision 0003 : la forme des pilotes d'équipements | `docs/decisions/0003-pilotes-d-equipements.md` |
| Étape 7 : le pilote du PCDU (`read_hk`, `set_switches`) et `DriverError`, 10 tests | `crates/sgr-flight/src/drivers/` et `tests/pcdu_driver.rs` |

Tout est au vert : `cargo test --workspace`, `cargo fmt --all -- --check`, `cargo lint` et `cargo check-nostd`.

## Reprendre sur une autre machine

```bash
git fetch
git switch claude/icd-v0-lockstep
cargo test --workspace
```

Résultat attendu, parmi des lignes à `0 passed` pour les cibles sans tests :

```
test result: ok. 10 passed; 0 failed     <- sgr-flight, tests/pcdu_driver.rs
test result: ok. 7 passed; 0 failed      <- sgr-flight-core, tests/platform.rs
test result: ok. 44 passed; 0 failed     <- sgr-protocol, tests unitaires
test result: ok. 2 passed; 0 failed      <- sgr-protocol, tests/reference_cycle.rs
```

Prérequis : `rustup` (la toolchain épinglée s'installe toute seule) et un compilateur C (`build-essential`). Node n'est pas nécessaire pour l'instant. Si `cargo` est introuvable dans un terminal, lancer `source "$HOME/.cargo/env"`.

## Ensuite

Il reste trois morceaux avant la fin de la base.

1. **La cadence du cycle et la boucle principale.** Rien n'est préparé, la forme est à discuter d'abord : un troisième trait de plateforme (attendre le `Tick`, envoyer le `CycleDone`), et la boucle qui, à chaque cycle, lit le housekeeping du PCDU et commande le réchauffeur selon la température. Questions à trancher : où vit la boucle (`sgr-flight-core` ou `sgr-flight`), quel seuil de température et quelle hystérésis, quoi faire quand le pilote renvoie une erreur.
2. **Le modèle du PCDU dans `sgr-sim`.** Le test `driver_and_pcdu_model_understand_each_other` en contient une ébauche (`PcduModel`).
3. **Le premier test de bout en bout dans `sgr-e2e`** : le simulateur et le logiciel de vol dans un seul processus. C'est la fin de la base : ensuite, l'auteur conçoit et écrit seul.

Une amélioration possible du pilote, laissée à l'auteur : `set_switches` donne au bus un buffer de `PcduHk::LEN` octets, alors que la réponse attendue est vide. Un buffer vide (`&mut []`) dirait mieux l'intention.

## Rappels

- **Le PCDU n'est pas la batterie.** C'est le boîtier électrique auquel elle est branchée : il la mesure, et il alimente chaque équipement par un interrupteur.
- **Un échange avec le PCDU** : le pilote envoie un `PcduRequest` encodé. `GET_HK` reçoit un `PcduHk`, `SET_SWITCHES` une réponse vide.
- **Une réponse vide n'est pas un silence** : `transfer` renvoie `Ok(0)` dans le premier cas, `Err(BusError::NoResponse)` dans le second.
- **Bus d'équipements et bus interne** sont deux choses. Le premier relie l'ordinateur de bord à ses équipements : seuls les pilotes y parlent. Le second relie les sous-systèmes entre eux, à l'intérieur du logiciel de vol.
- **Seuls les pilotes utilisent `sgr-protocol`** dans `sgr-flight`, et seulement son module `equipment`.
- **Une longueur qui vient de l'extérieur se vérifie avant de servir** : `&buffer[..length]` panique si `length` dépasse le buffer.

## Notions Rust déjà vues

Pour doser les explications : celles-ci sont acquises, les autres sont à présenter depuis zéro.

- `struct`, `impl`, constantes associées (`Self::LEN`), `derive`.
- `Result` et `Option`, et comment choisir entre les deux ; l'opérateur `?`.
- Les modules et `use`, y compris depuis une autre crate ; la visibilité (`pub` sur un type, sur chaque champ, sur une constante associée).
- Les `enum` simples et les `enum` qui portent des données ; `match`, y compris sur une tranche d'octets (`[CODE, value]`) et sur un `Result`.
- Les tableaux de taille fixe, `try_into`, `to_be_bytes` et `from_be_bytes`.
- `heapless::Vec` à travers l'alias `Payload` : `from_array`, `from_slice`.
- Déclarer un trait ; écrire une fonction générique sur un trait (`<B: EquipmentBus>`) ; `todo!()`.

Pas encore écrit par l'auteur : un `impl Trait for Type` (il en a lu dans les tests), une struct qui garde un état d'un cycle à l'autre.

## Méthode de travail suivie jusqu'ici

Pour une session Claude qui reprendrait sans l'historique de la conversation :

- L'auteur du projet débute en Rust (voir `CLAUDE.md`) : une notion à la fois, expliquée depuis zéro et rapprochée de TypeScript ou de JavaScript.
- **Le guidage est dégressif.** Il est serré pour la base, jusqu'au premier test de bout en bout, et se desserre d'un cran à chaque étape. Ensuite, l'auteur conçoit et écrit seul, tests compris : Claude relit, explique et génère le boilerplate à la demande.
- Une étape qui engage la conception commence par une proposition discutée, puis par une courte décision dans `docs/decisions/`.
- Claude prépare les tests de chaque étape **avant** le code. Il les valide hors du dépôt contre une implémentation jetable et contre quelques versions cassées exprès, puis ne laisse dans le dépôt que les tests.
- Tout nouveau type public est utilisé par un test du dossier `tests/` de sa crate, pour qu'un `pub` oublié casse la compilation.
- L'auteur écrit le code. Chaque étape se termine sur un résultat exact à obtenir (`N passed; 0 failed`), qu'il vérifie lui-même avant de demander la relecture.
- Claude relit en lançant les tests, `cargo lint`, `cargo fmt` et `cargo check-nostd`, et rattache chaque remarque au message du compilateur correspondant.
- Les commentaires de documentation (`///`) : l'auteur préfère que Claude les écrive, puis il les relit.
- L'auteur commite lui-même.
