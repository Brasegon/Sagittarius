# Travail en cours : étape 10, le PCDU côté simulateur

> **Note temporaire.** Elle sert à reprendre le travail sur une autre machine. Avant de fusionner la branche : supprimer ce fichier et la ligne de `CLAUDE.md` qui y renvoie (section « Avancement »).

## Où on en est

Branche `claude/icd-v0-lockstep`. Phase 1. Le protocole du bus est terminé dans `sgr-protocol`. Le logiciel de vol sait jouer un cycle complet face à une fausse plateforme. Le simulateur a son monde et son modèle du PCDU : il reste à écrire `Sim`, qui les assemble, puis viendra le premier test de bout en bout.

La vue d'ensemble du projet est dans `docs/guide.md`. Cette note ne couvre que l'étape en cours.

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
| Décision 0005 : la forme du simulateur minimal | `docs/decisions/0005-simulateur-minimal.md` |
| Squelette de `sgr-sim` : modules déclarés, en-têtes écrits, dépendance à `sgr-protocol` | `crates/sgr-sim/` |
| Tests de l'étape 10 : 12 tests pour le monde, 10 pour le PCDU, 13 pour le simulateur | `crates/sgr-sim/tests/` |
| Étape 10a : le monde (`World`), 12 tests au vert | `crates/sgr-sim/src/world.rs` |
| Étape 10b : le modèle du PCDU (`Pcdu`), 10 tests au vert | `crates/sgr-sim/src/equipment/pcdu.rs` |
| Le guide du projet | `docs/guide.md` |

**Il reste le morceau 10c, et une remarque de Clippy sur 10b.** `cargo test --workspace` et `cargo lint` ne compilent pas tant que `Sim` n'est pas écrit : ses 13 tests l'attendent. Les autres crates restent au vert, à vérifier une par une (`cargo test -p sgr-flight`). Un commit dans cet état est rouge et la CI échoue dessus : c'est voulu.

À corriger dans `Pcdu::handle` : Clippy signale `let_and_return`. Le résultat du `match` est rangé dans `decode`, puis renvoyé à la ligne suivante. Il suffit de supprimer `let decode = `, le `;` après l'accolade fermante du `match`, et la ligne `decode` : le `match` devient la dernière expression de la fonction.

## Reprendre sur une autre machine

```bash
git fetch
git switch claude/icd-v0-lockstep
cargo test --workspace
```

Prérequis : `rustup` (la toolchain épinglée s'installe toute seule) et un compilateur C (`build-essential`). Node n'est pas nécessaire pour l'instant. Si `cargo` est introuvable dans un terminal, lancer `source "$HOME/.cargo/env"`.

## Étape 10 : ce qu'il faut écrire

Les règles et leurs raisons sont dans la décision 0005. Les en-têtes des fichiers existent déjà, et `lib.rs` déclare les modules. Il y a trois morceaux, à écrire dans cet ordre. Chacun a son fichier de tests.

### 10a. Le monde, dans `crates/sgr-sim/src/world.rs`

- Cinq constantes publiques de type `f64`, avec les valeurs de la décision 0005 : `TICK_SECONDS`, `ENV_TEMP`, `BAT_HEAT_CAPACITY`, `BAT_THERMAL_RESISTANCE` et `BAT_HEATER_POWER`.
- La struct publique `World`, avec deux champs publics de type `f64` : `bat_temp`, en degrés Celsius, et `bat_v`, en volts. Son `derive` : `Debug, Clone, Copy, PartialEq`.
- La méthode publique `step(&mut self, heater_on: bool)`, qui avance la température de 100 ms avec la formule de la décision 0005.

```
cargo test -p sgr-sim --test world
test result: ok. 12 passed; 0 failed
```

### 10b. Le PCDU, dans `crates/sgr-sim/src/equipment/pcdu.rs`

- La struct publique `Pcdu`, avec un champ privé `switches: u8`. Même `derive` que `Thermostat`.
- `new() -> Self` : tous les interrupteurs ouverts.
- `heater_on(&self) -> bool` : l'état réel de l'interrupteur du réchauffeur.
- `handle(&mut self, world: &World, request: &[u8]) -> Option<Payload>` : décode la demande et y répond.
  - `GET_HK` : un `PcduHk` rempli avec les mesures du monde et l'état des interrupteurs, encodé.
  - `SET_SWITCHES` : remplace l'état des interrupteurs, et répond par un `Payload` vide.
  - Toute autre demande : `None`.

```
cargo test -p sgr-sim --test pcdu_model
test result: ok. 10 passed; 0 failed
```

### 10c. Le simulateur, dans `crates/sgr-sim/src/sim.rs`

- La struct publique `Sim` : deux champs publics, `world: World` et `pcdu: Pcdu`, et un champ privé `tick: u32`. Son `derive` : `Debug, Clone`.
- `new(world: World) -> Self` : le monde donné, un PCDU neuf, le tick 0.
- `tick(&self) -> u32` : le tick en cours.
- `bus_request(&mut self, address: u8, request: &[u8]) -> Option<Payload>` : donne la demande au PCDU si c'est son adresse. Sinon, `None`.
- `step(&mut self)` : avance le monde d'un pas, avec l'état du réchauffeur que donne le PCDU, puis passe au tick suivant.

```
cargo test -p sgr-sim --test sim
test result: ok. 13 passed; 0 failed
```

### Ce qui est nouveau

- **`f64`** : le nombre à virgule de Rust, l'équivalent du `number` de JavaScript. Un littéral s'écrit avec un point : `0.0`, pas `0`. Rust ne mélange jamais un entier et un flottant dans un calcul sans conversion explicite.
- **Pas de `Eq` avec un flottant.** `derive(Eq)` est refusé sur une struct qui contient un `f64`. `PartialEq` suffit pour `assert_eq!`.
- **Convertir avec `as`** : `x as i16` change un `f64` en entier. Il coupe les décimales, et sature aux bornes du type au lieu de reboucler. `x.round()` arrondit d'abord au plus proche.
- **Une struct qui en contient d'autres**, et **deux champs empruntés à la fois** : dans `bus_request`, `self.pcdu.handle(&self.world, request)` modifie un champ en lisant l'autre. Le compilateur l'accepte parce que ce sont deux champs distincts.
- **`match` sur une constante** : `PCDU_ADDRESS => ...` compare avec la valeur de la constante, et `_ => ...` prend tout le reste.

### Les pièges

- **Couper au lieu d'arrondir.** `0.29 * 100.0` vaut `28.999999999999996` : sans `round`, la mesure part à 28 au lieu de 29. Le test `measures_are_rounded_to_the_nearest_step` le détecte.
- **Les bits réservés.** L'ICD demande de les ignorer à la réception : le PCDU ne garde que les interrupteurs qui existent. Le test `reserved_bits_are_ignored` le détecte.
- **`new()` sans `Default`.** `cargo lint` refuse une struct qui a un `new()` sans argument et pas de `Default` (`clippy::new_without_default`). C'est pour cela que `Pcdu` reprend le `derive` de `Thermostat`.
- **Avancer le monde pendant le cycle.** Une demande sur le bus ne doit ni toucher à la température ni au tick. Les tests `world_does_not_move_during_the_cycle` et `bus_request_does_not_move_the_tick` le détectent.

### Résultat attendu

```
cargo test -p sgr-sim
test result: ok. 10 passed; 0 failed     <- tests/pcdu_model.rs
test result: ok. 13 passed; 0 failed     <- tests/sim.rs
test result: ok. 12 passed; 0 failed     <- tests/world.rs
```

Puis `cargo fmt --all`, `cargo lint` et `cargo check-nostd`, qui doivent passer tous les trois. À vérifier avant de demander la relecture.

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

Acquis avec les morceaux 10a et 10b : `f64`, la conversion avec `as` et `.round()`. En cours avec 10c : une struct qui en contient d'autres.

L'aide-mémoire complet, avec les équivalents TypeScript, est dans `docs/guide.md`, section 16.

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
