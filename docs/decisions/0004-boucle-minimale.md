# 0004 : boucle minimale du logiciel de vol

- **Date** : 2026-11-25
- **Statut** : acceptée

## Contexte

La phase 1 vise un squelette de bout en bout : le simulateur et le logiciel de vol qui tournent ensemble en lockstep. Le logiciel de vol a déjà ses traits de plateforme (décision 0002) et son premier pilote (décision 0003). Il lui manque de quoi jouer un cycle.

L'architecture visée par `docs/conception.md` est un ordonnanceur qui appelle des sous-systèmes par le trait `Subsystem`, reliés par un bus interne. Elle demande encore plusieurs décisions (sémantique du bus interne, chemin des mesures entre pilotes et sous-systèmes). On ne l'attend pas pour faire tourner un premier cycle.

## Décision

1. **Un troisième trait de plateforme, `CycleSync`**, dans `sgr_flight_core::platform` : `wait_tick` attend le début du cycle, `cycle_done` signale sa fin. En lockstep, ce sont les trames `Tick` et `CycleDone`. Sur la carte, `wait_tick` attendra un timer matériel.
2. **Le tick n'a qu'une source, `Clock`.** `wait_tick` ne renvoie rien : après lui, `Clock::tick` donne le numéro du cycle qui commence.
3. **Une seule fonction joue le cycle**, `sgr_flight::cycle::run_cycle`. Elle reçoit la plateforme, qui doit implémenter `CycleSync` et `EquipmentBus`, et le thermostat. Dans l'ordre :
   1. attendre le tick ;
   2. lire le housekeeping du PCDU ;
   3. donner la température au thermostat, si la lecture a réussi ;
   4. envoyer l'état voulu des interrupteurs ;
   5. signaler la fin du cycle.
4. **Un cycle va toujours jusqu'au bout.** Aucune erreur n'interrompt la fonction : sans `CycleDone`, le simulateur attendrait pour toujours. Les deux résultats, celui de la lecture et celui de la commande, reviennent à l'appelant dans un `CycleReport`.
5. **Sans mesure valide, le thermostat garde sa dernière commande.** On ne lui donne ni zéro ni valeur par défaut.
6. **L'état des interrupteurs est envoyé à chaque cycle**, même s'il n'a pas changé. Si le PCDU a redémarré, un cycle suffit à le rétablir.
7. **Le thermostat du réchauffeur de la batterie** : allumé en dessous de +5,00 °C, éteint au-dessus de +10,00 °C, inchangé entre les deux, éteint au démarrage.
8. **Le code au-dessus des pilotes n'importe pas `sgr-protocol`.** Le pilote du PCDU réexporte ce dont il a besoin : `PcduHk` et `SWITCH_BAT_HEATER`.

## Conséquences

Ce qu'on y gagne :

- Le logiciel de vol peut jouer un cycle complet, ce qui ouvre la voie au premier test de bout en bout.
- Le cycle se teste sans simulateur, face à une fausse plateforme qui note les appels reçus.
- La logique du thermostat est séparée du reste : elle se teste seule, et deviendra le cœur du sous-système thermique.

Ce que ça coûte :

- `run_cycle` sera remplacée quand l'ordonnanceur et le trait `Subsystem` existeront. Le thermostat, le pilote et les traits resteront.
- Deux échanges avec le PCDU à chaque cycle, dont un qui ne change le plus souvent rien.

Les écarts avec un vrai satellite :

- Les seuils du thermostat seraient des paramètres modifiables depuis le sol, pas des constantes.
- Un réchauffeur commandé à l'aveugle, faute de mesure, ne le resterait pas longtemps : au bout de quelques cycles, le FDIR prendrait la main (mesure redondante, repli en mode sûr). Ici, le thermostat garde sa commande indéfiniment.
- Une erreur d'équipement produirait un événement horodaté, descendu au sol. Ici, elle n'est que renvoyée à l'appelant.

Ce qui reste ouvert :

- **Les erreurs de la cadence.** `wait_tick` et `cycle_done` ne peuvent pas échouer. L'implémentation Linux dira si une erreur de socket doit remonter.
- **Ce que l'appelant fait du `CycleReport`** : il nourrira le service d'événements et le housekeeping, qui n'existent pas encore.
