# Travail en cours : étape 4, le housekeeping du PCDU

> **Note temporaire.** Elle sert à reprendre le travail sur une autre machine. Avant de fusionner la branche : supprimer ce fichier et la ligne de `CLAUDE.md` qui y renvoie (section « Avancement »).

## Où on en est

Branche `claude/icd-v0-lockstep`. Phase 1, premier morceau : le protocole entre le simulateur et le logiciel de vol.

| Fait | Où |
| ---- | -- |
| ICD v0.2 : dialogue par bus d'équipements, premier équipement le PCDU | `docs/icd.md` |
| Note de décision : pourquoi le dialogue par bus | `docs/decisions/0001-dialogue-par-bus-d-equipements.md` |
| Les quatre trames du bus, 32 tests | `crates/sgr-protocol/src/lockstep.rs` |
| Rangement et 6 tests pour le PCDU | `crates/sgr-protocol/src/equipment/` |

**Le dernier commit est volontairement rouge** : les tests du PCDU sont écrits, pas encore le code. `cargo test -p sgr-protocol` ne compile donc pas (21 erreurs sur `PcduHk`, `PCDU_ADDRESS` et `SWITCH_BAT_HEATER`), et la CI échoue sur ce commit. C'est l'étape 4 qui le remet au vert.

## Reprendre sur une autre machine

```bash
git fetch
git switch claude/icd-v0-lockstep
cargo test -p sgr-protocol
```

Prérequis : `rustup` (la toolchain épinglée s'installe toute seule) et un compilateur C (`build-essential`). Node n'est pas nécessaire pour cette étape.

## Étape 4 : ce qu'il faut écrire

Dans `crates/sgr-protocol/src/equipment/pcdu.rs`, entre l'en-tête `//!` et la ligne `mod tests;` :

1. **Deux constantes publiques** : `PCDU_ADDRESS`, qui vaut `0x10`, et `SWITCH_BAT_HEATER`, le bit 0 (`1 << 0`).
2. **La struct `PcduHk`** : `bat_v` en `u16`, `bat_temp` en `i16`, `switches` en `u8`. Taille fixe, donc `Copy` dans le `derive`.
3. **Son `impl`** : `LEN` à 5, puis :

```rust
pub fn encode(&self) -> [u8; Self::LEN]
pub fn decode(bytes: &[u8]) -> Option<Self>
```

Positions des champs, d'après l'ICD :

| Décalage | Champ      | Contenu                              |
| -------- | ---------- | ------------------------------------ |
| 0 et 1   | `bat_v`    | tension de la batterie, en mV        |
| 2 et 3   | `bat_temp` | température, en centièmes de °C      |
| 4        | `switches` | état réel des interrupteurs          |

Il n'y a pas d'en-tête ici : c'est le contenu du `payload`, que les trames du bus transportent sans le lire. Le bus joue le rôle d'HTTP, le PCDU celui du JSON dans le corps du message.

La seule nouveauté est `Option` à la place de `Result`. `decode` ne peut échouer que d'une façon, une longueur différente de 5, donc pas besoin d'un `enum` d'erreurs :

- `Some(Self { ... })` quand ça marche, là où on écrivait `Ok(Self { ... })` ;
- `None` quand la longueur est fausse, là où on écrivait `Err(...)`.

Modèle à suivre : `Tick::encode` et `Tick::decode` dans `lockstep.rs`, sans l'en-tête.

### Résultat attendu

```
test result: ok. 38 passed; 0 failed
```

Pour ne lancer que les tests du PCDU (6 tests) :

```bash
cargo test -p sgr-protocol pcdu
```

Puis `cargo fmt --all`, `cargo lint` et `cargo check-nostd`, qui doivent passer tous les trois.

## Rappels

- **Le PCDU n'est pas la batterie.** C'est le boîtier électrique auquel elle est branchée : il la mesure, et il alimente chaque équipement par un interrupteur. Pour avoir des nouvelles de la batterie, le logiciel de vol interroge le PCDU.
- **Savoir si le réchauffeur est allumé** : `(hk.switches & SWITCH_BAT_HEATER) != 0`.
- **Gros-boutiste partout** : `to_be_bytes()` dans un sens, `u16::from_be_bytes([a, b])` dans l'autre.

## Ensuite

1. **Étape 5** : les deux commandes du PCDU, `GET_HK` et `SET_SWITCHES`, dans un `enum PcduRequest`. Les tests sont à préparer.
2. Les traits de plateforme `Clock` et `EquipmentBus` dans `sgr-flight-core`.
3. Le premier pilote, celui du PCDU, dans `sgr-flight`, et une boucle qui lit la température et commande le réchauffeur.
4. Le modèle du PCDU dans `sgr-sim`, puis le premier test de bout en bout dans `sgr-e2e`.

## Méthode de travail suivie jusqu'ici

Pour une session Claude qui reprendrait sans l'historique de la conversation :

- L'auteur du projet débute en Rust (voir `CLAUDE.md`) : une notion à la fois, expliquée depuis zéro et rapprochée de TypeScript ou de JavaScript.
- Claude prépare les tests de chaque étape **avant** le code, à partir des trames de référence de l'ICD. Il les valide hors du dépôt contre une implémentation jetable et contre quelques versions cassées exprès, puis ne laisse dans le dépôt que les tests.
- L'auteur écrit le code. Chaque étape se termine sur un résultat exact à obtenir (`N passed; 0 failed`).
- Claude relit en lançant les tests, `cargo lint`, `cargo fmt` et `cargo check-nostd`, et rattache chaque remarque au message du compilateur correspondant.
- Pour une notion vraiment nouvelle, un exemple complet d'abord, puis l'auteur écrit le cas jumeau seul.
