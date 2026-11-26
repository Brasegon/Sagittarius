# Sagittarius

Mission de satellite simulée : le logiciel de vol du CubeSat Sagittarius-1 en Rust, un simulateur d'environnement et un segment sol web, du lancement à la montée d'orbite, propulsion comprise.

C'est un projet d'apprentissage du logiciel de vol embarqué.

```
ground/web (React)
     │  WebSocket
ground/gateway (Node + TypeScript)
     │  UDP : liaison radio et canal « banc de test »
sgr-sim (Rust) : orbite, énergie, thermique, propulsion, canal radio
     │  UDP en lockstep : bus d'équipements simulé
sgr-flight-linux (Rust) : logiciel de vol
```

## Documentation

| Document | Contenu |
| -------- | ------- |
| [`docs/guide.md`](docs/guide.md) | **Par où commencer.** Le projet expliqué, son état, comment le reprendre. |
| [`docs/conception.md`](docs/conception.md) | La conception détaillée et la feuille de route. |
| [`docs/icd.md`](docs/icd.md) | Le format de chaque message. |
| [`docs/decisions/`](docs/decisions/) | Les décisions d'architecture. |

## Démarrer

Prérequis : `rustup`, un compilateur C (`build-essential`) et Node 26.

```bash
cargo test --workspace
```

```bash
cargo lint
```

Les autres commandes sont dans le [guide](docs/guide.md#14-commandes-et-outils).

## État

Phase 1 en cours : le squelette de bout en bout. Le détail est dans le [guide](docs/guide.md#3-où-en-est-le-projet).

## Licence

MIT, voir [`LICENSE`](LICENSE).
