# 0001 : dialogue par bus d'équipements

- **Date** : 2026-11-25
- **Statut** : acceptée

## Contexte

L'ICD v0.1 faisait circuler une trame unique par cycle dans chaque sens : le simulateur envoyait toutes les mesures (`SensorFrame`), le logiciel de vol répondait avec toutes ses commandes (`ActuatorFrame`).

C'est le lockstep le plus simple, mais il s'écarte d'un vrai satellite sur un point qui structure tout le logiciel de vol : à bord, l'ordinateur ne reçoit pas ses mesures, il interroge ses équipements un par un sur des bus. Avec la trame unique, il n'y a ni pilotes d'équipements, ni équipement qui ne répond plus, alors que c'est la panne la plus courante à traiter par le FDIR.

Le projet sert à apprendre le logiciel de vol tel qu'il se pratique. La décision est prise avant d'écrire les traits de plateforme et le simulateur, donc au moment où elle coûte le moins.

## Décision

1. **Le logiciel de vol interroge chaque équipement** par un échange demande-réponse sur un bus d'équipements simulé (`BusRequest`, `BusResponse`). Il est le maître du bus.
2. **Le simulateur héberge un modèle par équipement**, qui interprète les demandes et construit les réponses.
3. **Le cycle est encadré par deux trames** : `Tick` (le top d'horloge, émis par le simulateur) et `CycleDone` (émis par le logiciel de vol). Le monde est figé entre les deux, puis avance d'un pas.
4. **L'absence de réponse est explicite** : statut `NO_RESPONSE`, sans délai mesuré à l'horloge de la machine. Le temps reste entièrement simulé.
5. **Le logiciel de vol gagne une couche de pilotes d'équipements**, entre les services du cœur et les sous-systèmes. Les pilotes sont rangés par équipement, les sous-systèmes par fonction. Le trait de plateforme `EquipmentBus` remplace `SensorBus`.
6. **Les formats de chaque équipement vivent dans `sgr-protocol`**, partagés par le pilote (côté vol) et par le modèle (côté simulateur).
7. **Un seul équipement pour commencer** : le `PCDU`, qui mesure la batterie et commande les interrupteurs.

Les formats sont dans l'ICD v0.2.

## Conséquences

Ce qu'on y gagne :

- Le logiciel de vol a la structure d'un vrai : acquisition par des pilotes, puis traitement par les sous-systèmes.
- Un équipement muet devient une erreur de bus que le pilote remonte, matière première du FDIR en phase 2.
- Le trait de bus a la forme de ceux d'`embedded-hal`, le standard Rust des pilotes embarqués.
- C'est ainsi que travaillent les bancs de validation de l'industrie : ils simulent chaque équipement derrière son bus.

Ce que ça coûte :

- Six trames par cycle pour le squelette, au lieu de deux. Le simulateur est plus riche.
- Les trames `SensorFrame` et `ActuatorFrame` de l'ICD v0.1 disparaissent ; l'interface lockstep passe en version 2.
- Le premier résultat de bout en bout arrive plus tard.
- Le pilote et le modèle partagent le même code d'encodage : une erreur symétrique passerait inaperçue. Les trames de référence de l'ICD servent de garde-fou.

Ce qui reste ouvert :

- Le chemin des mesures entre un pilote et les sous-systèmes (voir les questions ouvertes de `docs/conception.md`).
- La liaison radio : sans doute un équipement émetteur-récepteur sur le même bus, à trancher avec l'ICD v0.3.
