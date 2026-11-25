# ICD : Sagittarius

Interface Control Document : la source de vérité des formats de messages. Toute modification d'un message commence ici, puis passe dans `sgr-protocol`.

**Version 0.2.** Elle couvre le bus d'équipements simulé entre `sgr-sim` et le logiciel de vol, et l'interface de chaque équipement. La liaison sol-bord et le canal « banc de test » viennent ensuite : voir [Évolutions prévues](#évolutions-prévues).

## Vue d'ensemble

Le logiciel de vol ne reçoit pas ses mesures : il les demande. À chaque cycle, il interroge ses équipements un par un sur un bus, comme l'ordinateur de bord d'un vrai satellite. Le simulateur joue le bus et les équipements.

Ce document a donc deux niveaux :

- **Le bus** ([Interface lockstep](#interface-lockstep-sgr-sim--logiciel-de-vol)) : comment une demande et sa réponse voyagent, et comment le cycle est cadencé.
- **Les équipements** ([Équipements](#équipements)) : ce que chaque équipement comprend et ce qu'il répond. Le bus transporte ces octets sans les interpréter.

## Conventions

- **Ordre des octets** : gros-boutiste (big-endian, l'« ordre réseau »), comme dans le CCSDS.
- **Numérotation des bits** : le bit 0 est le bit de poids faible. Attention, les documents CCSDS numérotent dans l'autre sens (bit 0 = poids fort).
- **Types** : `u8`, `u16`, `u32` non signés ; `i16` signé, en complément à deux.
- **Grandeurs physiques** : entiers à échelle fixe (millivolts, centièmes de degré…), jamais de flottants sur le fil.
- **Champs et bits réservés** : émis à 0, ignorés à la réception.
- **Noms** : mnémoniques en `SCREAMING_SNAKE_CASE`, champs en `snake_case`, repris tels quels dans le code.
- **Temps** : un tick = un cycle de 100 ms. Le temps bord vaut `tick` × 100 ms ; le tick 0 est la séparation du lanceur.

## Inventaire

### Équipements

| Mnémonique | Adresse | Rôle                                                                      |
| ---------- | ------- | ------------------------------------------------------------------------- |
| `PCDU`     | `0x10`  | Boîtier d'alimentation : mesure la batterie et commande les interrupteurs |

### Capteurs

Chaque capteur et chaque actionneur appartient à un équipement, qui le mesure ou le commande, et à un sous-système, qui en est responsable dans le logiciel de vol. Un autre sous-système peut s'en servir : `BAT_TEMP` intéresse aussi l'énergie.

| Mnémonique | Équipement | Sous-système | Grandeur                   | Type  | Résolution | Plage codable        |
| ---------- | ---------- | ------------ | -------------------------- | ----- | ---------- | -------------------- |
| `BAT_V`    | `PCDU`     | Énergie      | Tension du bus batterie    | `u16` | 1 mV       | 0 à 65,535 V         |
| `BAT_TEMP` | `PCDU`     | Thermique    | Température de la batterie | `i16` | 0,01 °C    | −327,68 à +327,67 °C |

### Actionneurs

| Mnémonique   | Équipement | Sous-système | Rôle                       | Commande     | Bit de `switches` |
| ------------ | ---------- | ------------ | -------------------------- | ------------ | ----------------- |
| `BAT_HEATER` | `PCDU`     | Thermique    | Réchauffeur de la batterie | tout-ou-rien | 0                 |

## Interface lockstep (`sgr-sim` ↔ logiciel de vol)

### Transport

- UDP sur la boucle locale ; un datagramme contient exactement une trame.
- Le logiciel de vol écoute sur `127.0.0.1:20100` (valeur par défaut, configurable). Il répond à l'adresse source du premier datagramme reçu : le simulateur n'a pas besoin de port fixe.
- Le logiciel de vol se lance en premier. Il n'y a pas de retransmission : en ping-pong sur la boucle locale, la perte d'un datagramme est négligeable.
- Dans `sgr-e2e`, les mêmes échanges se font en mémoire, sans sockets.

### Déroulement d'un cycle

1. Le simulateur envoie un `Tick` portant le tick N : c'est le top d'horloge du bord.
2. Le logiciel de vol exécute son cycle. Chaque fois qu'il a besoin d'un équipement, il envoie une `BusRequest` et attend la `BusResponse`. Les échanges se font un par un, autant qu'il en faut.
3. Le logiciel de vol envoie un `CycleDone` portant le tick N.
4. Le simulateur avance le monde de 100 ms et passe au tick N + 1.

Le monde est figé pendant les étapes 1 à 3 : toutes les mesures lues pendant le cycle N sont celles de l'instant N × 100 ms. Une commande change l'état de l'équipement dès sa réception, et ses effets physiques s'appliquent à l'étape 4.

Le logiciel de vol est le maître du bus : le simulateur n'émet jamais de `BusResponse` sans `BusRequest`. Aucune attente n'a de limite de durée : on peut poser un point d'arrêt d'un côté sans désynchroniser l'autre. Le premier tick est le 0.

### En-tête commun (6 octets)

| Décalage | Taille | Champ        | Type  | Description                               |
| -------- | ------ | ------------ | ----- | ----------------------------------------- |
| 0        | 1      | `version`    | `u8`  | Version de l'interface lockstep : `2`     |
| 1        | 1      | `frame_type` | `u8`  | Type de trame, voir le tableau ci-dessous |
| 2        | 4      | `tick`       | `u32` | Numéro du cycle en cours                  |

| `frame_type` | Trame         | Sens             | Taille         |
| ------------ | ------------- | ---------------- | -------------- |
| `1`          | `Tick`        | simulateur → vol | 6 octets       |
| `2`          | `BusRequest`  | vol → simulateur | 7 à 39 octets  |
| `3`          | `BusResponse` | simulateur → vol | 7 à 39 octets  |
| `4`          | `CycleDone`   | vol → simulateur | 6 octets       |

`version` augmente à chaque changement incompatible d'une trame. La valeur 0 n'est valide ni pour `version` ni pour `frame_type` : un buffer rempli de zéros n'est pas une trame.

### `Tick` et `CycleDone`

Ces deux trames ne contiennent que l'en-tête.

### `BusRequest` (vol → simulateur)

| Décalage | Taille | Champ     | Type | Description                                         |
| -------- | ------ | --------- | ---- | --------------------------------------------------- |
| 0        | 6      | en-tête   |      | `frame_type` = `2`                                  |
| 6        | 1      | `address` | `u8` | Adresse de l'équipement interrogé                   |
| 7        | 0 à 32 | `payload` |      | Demande, dans le format de l'équipement destinataire |

### `BusResponse` (simulateur → vol)

| Décalage | Taille | Champ     | Type | Description                                      |
| -------- | ------ | --------- | ---- | ------------------------------------------------ |
| 0        | 6      | en-tête   |      | `frame_type` = `3`                               |
| 6        | 1      | `status`  | `u8` | `0` = `OK`, `1` = `NO_RESPONSE`                  |
| 7        | 0 à 32 | `payload` |      | Réponse de l'équipement ; vide si `NO_RESPONSE`  |

La longueur de `payload` est celle du reste du datagramme.

### Absence de réponse

Un équipement ne répond pas quand :

- aucun équipement n'a cette adresse ;
- l'équipement est en panne ;
- il ne comprend pas la demande.

Le simulateur envoie alors une `BusResponse` de statut `NO_RESPONSE`. Sur un vrai bus, il n'y aurait que du silence, constaté au bout d'un délai. Ici le silence est dit explicitement, pour que le temps reste entièrement simulé et les tests reproductibles.

### Règles de réception

Une trame est invalide si :

- sa longueur sort des bornes de son type ;
- `version` ne vaut pas `2` ;
- `frame_type` est inconnu, ou n'est pas celui attendu à ce moment de l'échange ;
- dans une `BusResponse`, `status` ne vaut ni `0` ni `1` ;
- `tick` n'est pas le tick en cours. Seule exception : pour le logiciel de vol, le tick d'un `Tick` fait foi, c'est lui qui donne le temps bord.

Une trame invalide est ignorée et le récepteur continue d'attendre.

## Équipements

### `PCDU` (adresse `0x10`)

Le boîtier d'alimentation (*Power Conditioning and Distribution Unit*). Le premier octet d'une demande est le code de la commande.

| Commande       | Code   | Demande (`payload`)              | Réponse (`payload`)         |
| -------------- | ------ | -------------------------------- | --------------------------- |
| `GET_HK`       | `0x01` | 1 octet : le code                | 5 octets : le housekeeping  |
| `SET_SWITCHES` | `0x02` | 2 octets : le code, `switches`   | vide                        |

Une demande dont le code est inconnu ou dont la longueur est fausse reste sans réponse.

#### Réponse à `GET_HK`

| Décalage | Taille | Champ      | Type  | Description                                              |
| -------- | ------ | ---------- | ----- | -------------------------------------------------------- |
| 0        | 2      | `bat_v`    | `u16` | `BAT_V`, en mV                                           |
| 2        | 2      | `bat_temp` | `i16` | `BAT_TEMP`, en centièmes de °C                           |
| 4        | 1      | `switches` | `u8`  | État réel des interrupteurs, même codage que la commande |

#### `switches`

Un bit par interrupteur : bit 0 pour `BAT_HEATER` (1 = fermé, le réchauffeur chauffe), bits 1 à 7 réservés.

`SET_SWITCHES` est une commande d'état : elle donne l'état voulu de tous les interrupteurs à la fois. Le `PCDU` l'applique dès réception, et un `GET_HK` envoyé ensuite dans le même cycle renvoie déjà le nouvel état. Relire `switches` permet au logiciel de vol de vérifier qu'une commande a bien été appliquée.

## Trames de référence

Les tests unitaires de `sgr-protocol` encodent et décodent ces trames à l'octet près. Les six premières forment un cycle complet, dans l'ordre.

| Trame         | Contenu                                                                       | Octets (hexadécimal)                  |
| ------------- | ----------------------------------------------------------------------------- | ------------------------------------- |
| `Tick`        | tick 1                                                                        | `02 01 00 00 00 01`                   |
| `BusRequest`  | tick 1, `PCDU`, `GET_HK`                                                      | `02 02 00 00 00 01 10 01`             |
| `BusResponse` | tick 1, `OK`, `BAT_V` 28 000 mV, `BAT_TEMP` −5,25 °C, tout ouvert             | `02 03 00 00 00 01 00 6D 60 FD F3 00` |
| `BusRequest`  | tick 1, `PCDU`, `SET_SWITCHES`, `BAT_HEATER` fermé                            | `02 02 00 00 00 01 10 02 01`          |
| `BusResponse` | tick 1, `OK`, vide                                                            | `02 03 00 00 00 01 00`                |
| `CycleDone`   | tick 1                                                                        | `02 04 00 00 00 01`                   |
| `BusResponse` | tick 74 565, `OK`, `BAT_V` 33 600 mV, `BAT_TEMP` 21,50 °C, `BAT_HEATER` fermé | `02 03 00 01 23 45 00 83 40 08 66 01` |
| `BusResponse` | tick 74 565, `NO_RESPONSE`                                                    | `02 03 00 01 23 45 01`                |

## Écarts avec un vrai satellite

- **Un seul bus, générique.** À bord, plusieurs bus cohabitent (I²C, SPI, CAN, MIL-STD-1553, SpaceWire), chacun avec son tramage et ses contraintes de temps. Le trait `EquipmentBus` de la plateforme isole cette simplification du reste du logiciel de vol.
- **Le silence est explicite.** Un équipement muet se constate à bord par un délai dépassé, pas par un statut `NO_RESPONSE`.
- **Le monde est figé pendant le cycle.** À bord, chaque échange prend du temps et les mesures d'un même cycle ne datent pas exactement du même instant.
- **Des unités physiques sur le fil.** Un capteur réel livre une valeur brute, en pas de convertisseur analogique-numérique, que le logiciel de vol ou le sol convertit avec une courbe d'étalonnage.
- **Pas de CRC sur les trames lockstep.** UDP a déjà sa somme de contrôle. La liaison série de la phase 5 ajoutera sous ces trames une couche de délimitation et de CRC, sans les modifier.

## Évolutions prévues

- **v0.3 (phase 1)** : liaison radio par le bus, sans doute sous la forme d'un équipement émetteur-récepteur (lecture des TC, écriture de la TM), ce qui demandera des `payload` plus longs ; enveloppe sol-bord (en-tête primaire CCSDS, CRC-16), premier paquet de housekeeping, paquets de référence partagés entre Rust et TypeScript, port de la liaison radio.
- **v1 (phase 2)** : protocole sol-bord complet (TC, ACK, HK, événements), pannes d'équipements (silence, mesures fausses, interrupteur bloqué), canal « banc de test ».
- **Phases 3 et 4** : équipements de la propulsion (pression et températures du réservoir, vannes, lit catalytique), puis de l'attitude.

## Historique

| Version | Date       | Changement                                                                                                                                                                          |
| ------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0.1     | 2026-11-25 | Version initiale : inventaire et interface lockstep, une trame par cycle dans chaque sens.                                                                                          |
| 0.2     | 2026-11-25 | Dialogue par bus d'équipements : `Tick`, `BusRequest`, `BusResponse`, `CycleDone` ; premier équipement, le `PCDU`. Voir `docs/decisions/0001-dialogue-par-bus-d-equipements.md`.     |
