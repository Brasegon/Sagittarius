# Travail en cours : le protocole du bus est terminé, les traits de plateforme viennent ensuite

> **Note temporaire.** Elle sert à reprendre le travail sur une autre machine. Avant de fusionner la branche : supprimer ce fichier et la ligne de `CLAUDE.md` qui y renvoie (section « Avancement »).

## Où on en est

Branche `claude/icd-v0-lockstep`. Phase 1, premier morceau : le protocole entre le simulateur et le logiciel de vol. Ce morceau est terminé côté `sgr-protocol`.

| Fait | Où |
| ---- | -- |
| ICD v0.2 : dialogue par bus d'équipements, premier équipement le PCDU | `docs/icd.md` |
| Note de décision : pourquoi le dialogue par bus | `docs/decisions/0001-dialogue-par-bus-d-equipements.md` |
| Les quatre trames du bus, 32 tests | `crates/sgr-protocol/src/lockstep.rs` |
| Étape 4 : le housekeeping du PCDU (`PcduHk`), 6 tests | `crates/sgr-protocol/src/equipment/pcdu.rs` |
| Étape 5 : les demandes au PCDU (`PcduRequest`), 6 tests | `crates/sgr-protocol/src/equipment/pcdu.rs` |
| Le cycle de référence de l'ICD rejoué par l'API publique, 2 tests | `crates/sgr-protocol/tests/reference_cycle.rs` |

Tout est au vert : `cargo test -p sgr-protocol`, `cargo fmt --all -- --check`, `cargo lint` et `cargo check-nostd`.

## Reprendre sur une autre machine

```bash
git fetch
git switch claude/icd-v0-lockstep
cargo test -p sgr-protocol
```

Résultat attendu, une ligne par cible :

```
test result: ok. 44 passed; 0 failed     <- tests unitaires
test result: ok. 2 passed; 0 failed      <- tests/reference_cycle.rs
```

Prérequis : `rustup` (la toolchain épinglée s'installe toute seule) et un compilateur C (`build-essential`). Node n'est pas nécessaire pour l'instant. Si `cargo` est introuvable dans un terminal, lancer `source "$HOME/.cargo/env"`.

## Ensuite

1. **Les traits de plateforme `Clock` et `EquipmentBus`** dans `sgr-flight-core`. Rien n'est préparé : la forme des traits est à discuter d'abord (ce que renvoie un échange sur le bus, comment un équipement muet est signalé).
2. Le premier pilote, celui du PCDU, dans `sgr-flight`, et une boucle qui lit la température et commande le réchauffeur.
3. Le modèle du PCDU dans `sgr-sim`, puis le premier test de bout en bout dans `sgr-e2e`.

## Rappels

- **Le PCDU n'est pas la batterie.** C'est le boîtier électrique auquel elle est branchée : il la mesure, et il alimente chaque équipement par un interrupteur.
- **Un échange avec le PCDU** : le pilote envoie un `PcduRequest` encodé dans le `payload` d'un `BusRequest`. `GET_HK` reçoit un `PcduHk`, `SET_SWITCHES` une réponse vide.
- **`SET_SWITCHES` est une commande d'état** : elle donne l'état voulu de tous les interrupteurs à la fois. Relire `switches` dans le housekeeping permet de vérifier qu'elle a été appliquée.
- **Savoir si le réchauffeur est allumé** : `(hk.switches & SWITCH_BAT_HEATER) != 0`.

## Notions Rust déjà vues

Pour doser les explications : celles-ci sont acquises, les autres sont à présenter depuis zéro.

- `struct`, `impl`, constantes associées (`Self::LEN`), `derive`.
- `Result` et `Option`, et comment choisir entre les deux ; l'opérateur `?`.
- Les modules et `use` ; la visibilité (`pub` sur un type, sur chaque champ, sur une constante associée).
- Les `enum` simples et les `enum` qui portent des données ; `match`, y compris sur une tranche d'octets (`[CODE, value]`).
- Les tableaux de taille fixe, `try_into`, `to_be_bytes` et `from_be_bytes`.
- `heapless::Vec` à travers l'alias `Payload` : `from_array`, `from_slice`.

Pas encore vus : les traits, qui sont la notion nouvelle de l'étape suivante.

## Méthode de travail suivie jusqu'ici

Pour une session Claude qui reprendrait sans l'historique de la conversation :

- L'auteur du projet débute en Rust (voir `CLAUDE.md`) : une notion à la fois, expliquée depuis zéro et rapprochée de TypeScript ou de JavaScript.
- Claude prépare les tests de chaque étape **avant** le code, à partir des trames de référence de l'ICD. Il les valide hors du dépôt contre une implémentation jetable et contre quelques versions cassées exprès, puis ne laisse dans le dépôt que les tests.
- Tout nouveau type public est aussi utilisé dans `crates/sgr-protocol/tests/reference_cycle.rs`, pour qu'un `pub` oublié casse la compilation.
- L'auteur écrit le code. Chaque étape se termine sur un résultat exact à obtenir (`N passed; 0 failed`).
- Claude relit en lançant les tests, `cargo lint`, `cargo fmt` et `cargo check-nostd`, et rattache chaque remarque au message du compilateur correspondant.
- Pour une notion vraiment nouvelle, un exemple complet d'abord, puis l'auteur écrit le cas jumeau seul.
- L'auteur commite lui-même.
