# 0003 : forme des pilotes d'équipements

- **Date** : 2026-11-25
- **Statut** : acceptée

## Contexte

La décision 0001 a créé une couche de pilotes d'équipements dans `sgr-flight`, et rangé les formats de chaque équipement dans `sgr-protocol`. La décision 0002 a fixé le trait `EquipmentBus`. Il reste à dire à quoi ressemble un pilote, avant d'écrire le premier, celui du PCDU.

Un point bloquait : `crates/sgr-flight/Cargo.toml` interdisait de dépendre de `sgr-protocol`, pour que les sous-systèmes ne voient aucun format de message. Or un pilote a besoin des formats de son équipement.

## Décision

1. **Un module par équipement**, dans `sgr_flight::drivers` : `drivers::pcdu` pour commencer.
2. **`sgr-flight` dépend de `sgr-protocol`, pour les pilotes seulement.** Seul le code de `src/drivers/` l'utilise, et seulement son module `equipment`. Les sous-systèmes ne voient toujours aucun format.
3. **Un pilote est un jeu de fonctions, une par commande de l'ICD.** Pour le PCDU : `read_hk` et `set_switches`. Chacune encode la demande, fait un échange, vérifie la réponse et la décode.
4. **Le bus est prêté le temps de l'appel** : chaque fonction est générique sur le bus et reçoit `&mut B`, avec `B: EquipmentBus`. Le pilote ne possède pas le bus.
5. **Une erreur commune à tous les pilotes**, `drivers::DriverError`, à deux variantes :
   - `NoResponse` : l'équipement est resté muet ;
   - `InvalidResponse` : il a répondu, mais la réponse est inutilisable (mauvaise longueur, ou trop longue pour le buffer du pilote).
6. **Un échange par appel, jamais de nouvel essai.** Réessayer est une décision du FDIR, pas du pilote.
7. **Un pilote ne fait pas confiance à la longueur que renvoie le bus** : il la vérifie avant de s'en servir. Un bus fautif ne doit pas pouvoir arrêter le logiciel de vol.

## Conséquences

Ce qu'on y gagne :

- Plusieurs pilotes se partagent le même bus, l'un après l'autre. Le compilateur vérifie qu'ils ne s'en servent pas en même temps.
- Un pilote se teste sans simulateur, face à un faux bus de quelques lignes.
- Le FDIR pourra compter les échecs de la même façon pour chaque équipement, puisque tous les pilotes renvoient le même type d'erreur.
- Le pilote traduit : au-dessus de lui, plus personne ne parle d'octets ni d'erreurs de bus.

Ce que ça coûte :

- `DriverError` ne garde pas le détail de l'erreur du bus. `ResponseTooLong` et une réponse trop courte donnent le même `InvalidResponse`.
- La règle « seuls les pilotes utilisent `sgr-protocol` » n'est pas vérifiée par le compilateur. Depuis le 2026-11-26, un test la vérifie à sa place : `crates/sgr-flight/tests/architecture.rs`.

Les écarts avec un vrai satellite :

- Un vrai pilote garde un état : la santé de l'équipement, des compteurs d'erreurs, la date de la dernière mesure valide. Ces données partent en télémesure et alimentent le FDIR. Ici, le pilote n'a pas de mémoire. Quand il en aura besoin, en phase 2, les fonctions deviendront les méthodes d'une struct.
- Un vrai pilote vérifie aussi l'intégrité de la réponse (CRC, parité). Le bus simulé n'en a pas.

Ce qui reste ouvert :

- **Ce que voient les sous-systèmes.** Le pilote renvoie `PcduHk` tel quel. Le chemin d'une mesure entre un pilote et les sous-systèmes reste une question ouverte de `docs/conception.md`.
- **La vérification d'une commande.** Relire `switches` après un `SET_SWITCHES` permet de détecter un interrupteur bloqué. Ce sera le travail du FDIR.
