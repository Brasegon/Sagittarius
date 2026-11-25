# Travail en cours : étape 9, le trait de cadence et le cycle

> **Note temporaire.** Elle sert à reprendre le travail sur une autre machine. Avant de fusionner la branche : supprimer ce fichier et la ligne de `CLAUDE.md` qui y renvoie (section « Avancement »).

## Où on en est

Branche `claude/icd-v0-lockstep`. Phase 1. Le protocole du bus est terminé dans `sgr-protocol`. Le logiciel de vol a ses traits de plateforme, son premier pilote et son thermostat : il lui manque de quoi jouer un cycle.

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
| Tests de l'étape 9 : 2 tests ajoutés pour le trait, 7 tests pour le cycle | `crates/sgr-flight-core/tests/platform.rs` et `crates/sgr-flight/tests/cycle.rs` |

**Les tests de l'étape 9 sont écrits, pas encore le code.** `cargo test -p sgr-flight-core` et `cargo test -p sgr-flight` ne compilent donc pas : il manque `CycleSync`, `CycleReport`, `run_cycle` et deux réexports dans le pilote. `sgr-protocol` reste au vert. Si ces tests sont commités avant le code, ce commit est rouge et la CI échoue dessus : c'est voulu.

## Reprendre sur une autre machine

```bash
git fetch
git switch claude/icd-v0-lockstep
cargo test --workspace
```

Prérequis : `rustup` (la toolchain épinglée s'installe toute seule) et un compilateur C (`build-essential`). Node n'est pas nécessaire pour l'instant. Si `cargo` est introuvable dans un terminal, lancer `source "$HOME/.cargo/env"`.

## Étape 9 : ce qu'il faut écrire

Les règles et leurs raisons sont dans la décision 0004. Il y a trois endroits à toucher.

**1. Dans `crates/sgr-flight-core/src/platform.rs`** : le trait public `CycleSync`, avec deux méthodes.

```rust
fn wait_tick(&mut self);
fn cycle_done(&mut self);
```

**2. Dans `crates/sgr-flight/src/drivers/pcdu.rs`** : réexporter `PcduHk` et `SWITCH_BAT_HEATER`, pour que le code au-dessus du pilote n'ait pas à importer `sgr-protocol`. Un réexport s'écrit `pub use` à la place de `use`. C'est l'équivalent de `export { X } from "./y"` en TypeScript.

**3. Dans `crates/sgr-flight/src/cycle.rs`**, dont l'en-tête existe déjà :

- La struct publique `CycleReport`, avec deux champs publics : `acquisition`, un `Result<PcduHk, DriverError>`, et `command`, un `Result<(), DriverError>`. Même `derive` que `DriverError`.
- La fonction publique `run_cycle` :

```rust
pub fn run_cycle<P: CycleSync + EquipmentBus>(
    platform: &mut P,
    thermostat: &mut Thermostat,
) -> CycleReport
```

Ce qu'elle fait, dans l'ordre :

1. Attendre le tick.
2. Lire le housekeeping du PCDU, avec le pilote.
3. Si la lecture a réussi, donner `bat_temp` au thermostat. Sinon, ne pas y toucher.
4. Envoyer l'état des interrupteurs, avec le pilote : `SWITCH_BAT_HEATER` si le thermostat veut chauffer, `0` sinon. À chaque cycle, même si la lecture a échoué.
5. Signaler la fin du cycle.
6. Renvoyer les deux résultats dans un `CycleReport`.

### Ce qui est nouveau

- **`P: CycleSync + EquipmentBus`** : le `+` demande un type qui implémente les deux traits. Une seule plateforme donne la cadence et le bus, parce qu'en lockstep une seule socket transporte tout.
- **Prêter la plateforme au pilote** : `platform` est déjà un `&mut P`, on le passe tel quel à `read_hk` et à `set_switches`. Le prêt se termine au retour de chaque appel.
- **Garder un `Result` au lieu de le traiter tout de suite** : ici, les deux `Result` sont rangés dans le rapport, erreur ou non.

### Le piège : `?` arrête le cycle

`let hk = pcdu::read_hk(platform)?;` est l'écriture qui vient naturellement. Mais `?` fait sortir la fonction dès la première erreur : le cycle s'arrêterait avant `cycle_done`, et le simulateur attendrait pour toujours. Le test `cycle_ends_even_when_the_pcdu_is_silent` le détecte. De toute façon, `?` ne compilerait pas ici, puisque `run_cycle` ne renvoie pas un `Result`.

### Résultat attendu

```
cargo test -p sgr-flight-core
test result: ok. 9 passed; 0 failed      <- tests/platform.rs

cargo test -p sgr-flight
test result: ok. 7 passed; 0 failed      <- tests/cycle.rs
test result: ok. 10 passed; 0 failed     <- tests/pcdu_driver.rs
test result: ok. 11 passed; 0 failed     <- tests/thermal.rs
```

Puis `cargo fmt --all`, `cargo lint` et `cargo check-nostd`, qui doivent passer tous les trois. À vérifier avant de demander la relecture.

## Ensuite

Il reste deux morceaux avant la fin de la base.

1. **Le modèle du PCDU dans `sgr-sim`.** Les tests en contiennent deux ébauches : `PcduModel` dans `tests/pcdu_driver.rs` et `FakePlatform` dans `tests/cycle.rs`.
2. **Le premier test de bout en bout dans `sgr-e2e`** : le simulateur et le logiciel de vol dans un seul processus. L'auteur y écrira son premier `impl Trait for Type`, pour brancher la plateforme sur le simulateur. C'est la fin de la base : ensuite, il conçoit et écrit seul.

Deux améliorations possibles, laissées à l'auteur :

- `set_switches` donne au bus un buffer de `PcduHk::LEN` octets, alors que la réponse attendue est vide. Un buffer vide (`&mut []`) dirait mieux l'intention.
- Les seuils `500` et `1000` du thermostat sont écrits en dur. Deux constantes nommées diraient ce qu'ils valent et dans quelle unité.

## Rappels

- **Le PCDU n'est pas la batterie.** C'est le boîtier électrique auquel elle est branchée : il la mesure, et il alimente chaque équipement par un interrupteur.
- **Un échange avec le PCDU** : le pilote envoie un `PcduRequest` encodé. `GET_HK` reçoit un `PcduHk`, `SET_SWITCHES` une réponse vide.
- **Une réponse vide n'est pas un silence** : `transfer` renvoie `Ok(0)` dans le premier cas, `Err(BusError::NoResponse)` dans le second.
- **Bus d'équipements et bus interne** sont deux choses. Le premier relie l'ordinateur de bord à ses équipements : seuls les pilotes y parlent. Le second relie les sous-systèmes entre eux, à l'intérieur du logiciel de vol.
- **Seuls les pilotes utilisent `sgr-protocol`** dans `sgr-flight`, et seulement son module `equipment`.
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

En cours avec l'étape 9 : deux traits dans une borne (`P: A + B`), `pub use`.

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
