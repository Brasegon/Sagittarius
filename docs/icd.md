# ICD : Sagittarius

Interface Control Document : la source de vérité des formats de messages. Toute modification d'un message commence ici, puis passe dans `sgr-protocol`.

**Version 0.1.** La v0 couvre l'inventaire des capteurs et des actionneurs, et l'interface lockstep entre `sgr-sim` et le logiciel de vol. La liaison sol-bord et le canal « banc de test » viennent ensuite : voir [Évolutions prévues](#évolutions-prévues).

## Conventions

- **Ordre des octets** : gros-boutiste (big-endian, l'« ordre réseau »), comme dans le CCSDS.
- **Numérotation des bits** : le bit 0 est le bit de poids faible. Attention, les documents CCSDS numérotent dans l'autre sens (bit 0 = poids fort).
- **Types** : `u8`, `u16`, `u32` non signés ; `i16` signé, en complément à deux.
- **Grandeurs physiques** : entiers à échelle fixe (millivolts, centièmes de degré…), jamais de flottants sur le fil.
- **Champs et bits réservés** : émis à 0, ignorés à la réception.
- **Noms** : mnémoniques en `SCREAMING_SNAKE_CASE`, champs en `snake_case`, repris tels quels dans le code.
- **Temps** : un tick = un cycle de 100 ms. Le temps bord vaut `tick` × 100 ms ; le tick 0 est la séparation du lanceur.

## Inventaire

Chaque capteur et chaque actionneur est rattaché à un seul sous-système, celui qui en est responsable. Les autres peuvent s'en servir : `BAT_TEMP` intéresse aussi l'énergie. `sgr-protocol` suit ce découpage, avec un fichier par sous-système.

### Capteurs

| Mnémonique | Sous-système | Grandeur                   | Type  | Résolution | Plage codable        |
| ---------- | ------------ | -------------------------- | ----- | ---------- | -------------------- |
| `BAT_V`    | Énergie      | Tension du bus batterie    | `u16` | 1 mV       | 0 à 65,535 V         |
| `BAT_TEMP` | Thermique    | Température de la batterie | `i16` | 0,01 °C    | −327,68 à +327,67 °C |

### Actionneurs

| Mnémonique   | Sous-système | Rôle                       | Commande     | Bit de `switches` |
| ------------ | ------------ | -------------------------- | ------------ | ----------------- |
| `BAT_HEATER` | Thermique    | Réchauffeur de la batterie | tout-ou-rien | 0                 |

## Interface lockstep (`sgr-sim` ↔ logiciel de vol)

### Transport

- UDP sur la boucle locale ; un datagramme contient exactement une trame.
- Le logiciel de vol écoute sur `127.0.0.1:20100` (valeur par défaut, configurable). Il répond à l'adresse source du datagramme reçu : le simulateur n'a pas besoin de port fixe.
- Le logiciel de vol se lance en premier. Il n'y a pas de retransmission : en ping-pong sur la boucle locale, la perte d'un datagramme est négligeable.
- Dans `sgr-e2e`, les mêmes messages circulent en mémoire, sans sockets.

### Déroulement d'un cycle

1. Le simulateur envoie une `SensorFrame` portant le tick N : les mesures à l'instant N × 100 ms.
2. Le logiciel de vol exécute son cycle, puis répond par une `ActuatorFrame` qui recopie le tick N.
3. Le simulateur applique les commandes, avance le monde de 100 ms et passe au tick N + 1.

Le simulateur attend la réponse sans limite de durée : on peut poser un point d'arrêt dans le logiciel de vol sans désynchroniser le monde. Le premier tick est le 0.

### En-tête commun (6 octets)

| Décalage | Taille | Champ        | Type  | Description                                                    |
| -------- | ------ | ------------ | ----- | -------------------------------------------------------------- |
| 0        | 1      | `version`    | `u8`  | Version de l'interface lockstep : `1`                          |
| 1        | 1      | `frame_type` | `u8`  | `1` = `SensorFrame`, `2` = `ActuatorFrame`                     |
| 2        | 4      | `tick`       | `u32` | Numéro du cycle                                                |

`version` augmente à chaque changement incompatible d'une trame. La valeur 0 n'est valide ni pour `version` ni pour `frame_type` : un buffer rempli de zéros n'est pas une trame.

### `SensorFrame` (simulateur → vol, 10 octets)

| Décalage | Taille | Champ      | Type  | Description                           |
| -------- | ------ | ---------- | ----- | ------------------------------------- |
| 0        | 6      | en-tête    |       | `frame_type` = `1`                    |
| 6        | 2      | `bat_v`    | `u16` | `BAT_V`, en mV                        |
| 8        | 2      | `bat_temp` | `i16` | `BAT_TEMP`, en centièmes de °C        |

### `ActuatorFrame` (vol → simulateur, 7 octets)

| Décalage | Taille | Champ      | Type | Description                                             |
| -------- | ------ | ---------- | ---- | ------------------------------------------------------- |
| 0        | 6      | en-tête    |      | `frame_type` = `2`                                      |
| 6        | 1      | `switches` | `u8` | Bit 0 : `BAT_HEATER` (1 = allumé). Bits 1 à 7 réservés. |

`switches` est une commande d'état : le logiciel de vol renvoie à chaque cycle l'état voulu de tous les interrupteurs, et le simulateur l'applique pendant tout l'intervalle entre le tick N et le tick N + 1.

### Règles de réception

Une trame est invalide si :

- sa longueur n'est pas exactement celle de son type ;
- `version` ne vaut pas `1` ;
- `frame_type` est inconnu, ou n'est pas celui attendu dans ce sens ;
- côté simulateur, `tick` n'est pas le tick en cours.

Une trame invalide est ignorée et le récepteur continue d'attendre. Côté logiciel de vol, le tick reçu fait foi : c'est lui qui donne le temps bord.

### Trames de référence

Les tests unitaires de `sgr-protocol` encodent et décodent ces trames à l'octet près.

| Trame           | Contenu                                                | Octets (hexadécimal)            |
| --------------- | ------------------------------------------------------ | ------------------------------- |
| `SensorFrame`   | tick 1, `BAT_V` 28 000 mV, `BAT_TEMP` −5,25 °C         | `01 01 00 00 00 01 6D 60 FD F3` |
| `SensorFrame`   | tick 74 565, `BAT_V` 33 600 mV, `BAT_TEMP` 21,50 °C    | `01 01 00 01 23 45 83 40 08 66` |
| `ActuatorFrame` | tick 1, `BAT_HEATER` allumé                            | `01 02 00 00 00 01 01`          |
| `ActuatorFrame` | tick 74 565, tout éteint                               | `01 02 00 01 23 45 00`          |

## Écarts avec un vrai satellite

- **Une trame unique par cycle.** À bord, chaque équipement a son bus (I²C, SPI, CAN, MIL-STD-1553, SpaceWire) et son propre ICD. Le trait `SensorBus` de la plateforme isole cette simplification du reste du logiciel de vol.
- **Des unités physiques sur le fil.** Un capteur réel livre une valeur brute, en pas de convertisseur analogique-numérique, que le logiciel de vol ou le sol convertit avec une courbe d'étalonnage.
- **Pas de CRC sur les trames lockstep.** UDP a déjà sa somme de contrôle. La liaison série de la phase 5 ajoutera sous ces trames une couche de délimitation et de CRC, sans les modifier.

## Évolutions prévues

- **v0.2 (phase 1)** : section radio en fin de trame dans les deux sens (TC remises sur un tick précis, TM émises), enveloppe sol-bord (en-tête primaire CCSDS, CRC-16), premier paquet de housekeeping, paquets de référence partagés entre Rust et TypeScript, port de la liaison radio.
- **v1 (phase 2)** : protocole sol-bord complet (TC, ACK, HK, événements), validité des mesures, canal « banc de test » et injection de pannes.
- **Phases 3 et 4** : capteurs et actionneurs de la propulsion (pression et températures du réservoir, vannes, lit catalytique), puis de l'attitude.

## Historique

| Version | Date       | Changement                                                    |
| ------- | ---------- | ------------------------------------------------------------- |
| 0.1     | 2026-11-25 | Version initiale : inventaire v0 et interface lockstep.       |
