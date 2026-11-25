# Conception : Sagittarius

## Mission

Le satellite Sagittarius-1 est largué par un lanceur sur une orbite d'injection à environ 400 km. Il traverse la LEOP (Launch and Early Orbit Phase), puis monte par ses propres moyens jusqu'à son orbite opérationnelle à environ 550 km.

Scénario LEOP :

1. Séparation du lanceur : le logiciel de vol démarre en mode SAFE.
2. Réduction de la rotation (detumbling).
3. Déploiement des panneaux solaires.
4. Premier contact radio avec la station sol.
5. Recette des sous-systèmes, passage en mode NOMINAL.
6. Montée d'orbite par transfert de Hohmann (deux allumages, environ 85 m/s au total).
7. Passage en mode MISSION.

## Simulateur (`sgr-sim`)

Le « monde » autour du satellite, équipements et canal radio compris. Il ne connaît du logiciel de vol que le bus d'équipements, et relaie les paquets radio sans les décoder.

- **Lockstep** : le simulateur donne le tick. À chaque cycle, il annonce le tick, répond aux demandes que le logiciel de vol adresse aux équipements, puis avance d'un pas quand celui-ci signale la fin de son cycle (voir `docs/decisions/0001-dialogue-par-bus-d-equipements.md`). Le temps bord vient du tick, jamais de l'horloge de la machine. Les tests sont reproductibles et peuvent tourner plus vite que le temps réel. Un mode temps réel sert aux démos.
- **Liaison radio** : le simulateur joue le canal RF entre la passerelle et le logiciel de vol. Il relaie les paquets pendant les fenêtres de visibilité, les jette en dehors, et remet chaque TC au logiciel de vol sur un tick précis, ce qui garde les tests reproductibles. Les simulateurs opérationnels font de même : le centre de contrôle s'y connecte comme à une station sol.
- **Bibliothèque et binaire** : les tests de bout en bout exécutent le simulateur et le logiciel de vol dans un seul processus, sans sockets. Le binaire sert aux démos, en UDP.
- **Orbite** : intégration numérique (deux corps, puis J2 et traînée), avec application des poussées. SGP4 ne gère pas les manœuvres, il ne sert qu'à initialiser depuis un TLE réel si besoin. Calcul des éclipses et des fenêtres de visibilité de la station sol.
- **Énergie** : production des panneaux (éclairement, attitude), état de charge de la batterie, consommation par équipement.
- **Thermique** : modèle simple par nœuds (réservoir, lignes, lit catalytique, batterie, électronique) et réchauffeurs.
- **Propulsion** : voir la section dédiée.
- **Attitude** : parfaite au début (le satellite pointe où on le lui demande), modèle simplifié ensuite.
- **Équipements** : un modèle par équipement (le PCDU d'abord), que le logiciel de vol interroge sur un bus simulé. Mesures bruitées, pannes possibles, dont l'absence de réponse. L'inventaire des équipements, des capteurs et des actionneurs est fixé dans l'ICD.
- **Injection de pannes** : pilotée depuis l'interface par le canal « banc de test », ou scriptée dans un scénario (panne à un instant donné) pour les tests reproductibles.

## Logiciel de vol

### Couches

1. **Plateforme** (traits dans `sgr-flight-core`, implémentation dans `sgr-flight-linux`) : `Clock`, `GroundLink`, `EquipmentBus`… C'est l'équivalent de l'OSAL et du PSP de cFS, et la seule partie à réécrire pour la STM32. `Clock` fournit le temps bord : en lockstep il avance avec le tick du simulateur, sur cible avec un timer matériel.
2. **Services du cœur** (`sgr-flight-core`) :
   - **Ordonnanceur** : exécutif cyclique à 10 Hz, cadences lentes par diviseurs.
   - **Bus interne** : files à capacité fixe (`heapless`), seul moyen d'échange entre sous-systèmes.
   - **Paramètres** : valeurs modifiables depuis le sol sans recompiler (seuils du FDIR, cadences…), l'équivalent des tables cFS.
   - **Événements** : journal horodaté avec niveaux de gravité.
   - **TC/TM** : décodage, validation, aiguillage et acquittement des TC ; encodage de la télémesure ; file des commandes temporisées.
   - **Stockage de la télémesure** : tampon circulaire de taille fixe qui conserve le housekeeping et les événements produits hors visibilité, vidé sur TC pendant un passage (l'équivalent du service 15 du PUS). Le logiciel de vol ne sait pas s'il est en visibilité : il émet la télémesure en continu et c'est le sol qui demande le vidage. Simplification : un vrai satellite n'allume son émetteur que pendant les passages.
3. **Pilotes d'équipements** (`sgr-flight`) : un pilote par équipement (le PCDU d'abord). Il dialogue avec son équipement par le trait `EquipmentBus`, traduit les octets en mesures et signale un équipement qui ne répond plus. Seuls les pilotes parlent au bus d'équipements, à ne pas confondre avec le bus interne des services du cœur.
4. **Sous-systèmes** (`sgr-flight`) : énergie, thermique, propulsion, attitude, modes et FDIR. Les modes et le FDIR implémentent le même trait que les autres ; ils sont simplement appelés en dernier.

### Interface des sous-systèmes

```rust
pub trait Subsystem {
    /// Appelé à chaque cycle, dans un ordre fixe.
    fn step(&mut self, ctx: &mut Context);
    /// Télécommande destinée à ce sous-système.
    /// `Ok(Completed)` : exécutée sur-le-champ.
    /// `Ok(InProgress)` : acceptée, la fin sera rapportée plus tard via `ctx`.
    fn handle(&mut self, cmd: &Command, ctx: &mut Context) -> Result<CmdStatus, CmdError>;
    /// Remplit son paquet de housekeeping.
    fn housekeeping(&self, out: &mut HkWriter);
}
```

Le `Context` donne accès au bus, au temps, aux paramètres et aux événements, jamais aux autres sous-systèmes.

Une `Command` porte son identifiant de requête, opaque pour le sous-système. Une commande longue (allumage, déploiement) répond `InProgress`, puis rapporte sa fin par le `Context` avec cet identifiant : le service TC/TM en tire l'ACK « terminée » ou « échouée ». C'est le principe du service 1 du PUS (vérification des requêtes).

### Un cycle (100 ms)

1. Acquisition : les pilotes interrogent les équipements.
2. Traitement des TC reçues et des commandes temporisées arrivées à échéance.
3. `step()` de chaque sous-système, dans un ordre fixe.
4. `step()` des modes et du FDIR, toujours en dernier (repli en SAFE si nécessaire).
5. Commandes aux équipements, housekeeping, événements.

La durée de chaque cycle est mesurée avec l'horloge de la machine, distincte du temps bord : un dépassement émet un événement. Cette détection est désactivée quand le simulateur tourne en accéléré, sinon la charge du PC rendrait les tests non reproductibles.

## Protocole sol-bord

Protocole maison, défini en détail dans `docs/icd.md`.

- **Enveloppe** : en-tête primaire CCSDS Space Packet (6 octets), CRC-16 en fin de paquet.
- **Types de messages** :
  - **TC** : identifiant de commande, identifiant de requête, date d'exécution optionnelle, arguments. Une TC datée est stockée dans la file des commandes temporisées.
  - **ACK** : identifiant de requête, statut (acceptée, refusée, terminée, échouée), code d'erreur.
  - **HK** : sous-système, temps bord, valeurs.
  - **Événement** : gravité, identifiant, temps bord, paramètres.
- L'identifiant de requête permet au sol de savoir exactement quelle commande a réussi ou échoué.
- Les sous-systèmes ne voient jamais ce format : un passage au PUS ne changerait que le service TC/TM.
- **Paquets de référence** : un jeu de paquets encodés, versionné avec l'ICD, est vérifié par les tests Rust et TypeScript. Le protocole étant implémenté deux fois, c'est ce qui détecte une divergence. Écart avec l'industrie : une base TM/TC lisible par machine (XTCE, MIB) y sert de source unique aux deux côtés.

## Propulsion

Propulsion chimique monergol (hydrazine), réservoir en blowdown.

### Modèle physique (`sgr-sim`)

- Tsiolkovski : Δv = Isp · g₀ · ln(m₀ / m_f). La masse du satellite diminue à chaque allumage.
- Blowdown : le gaz pressurisant se détend avec la consommation, la pression baisse et la poussée avec elle.
- Thermique : l'hydrazine gèle vers 1,4 °C, donc réchauffeurs permanents sur le réservoir et les lignes. Le lit catalytique doit être préchauffé avant chaque allumage.
- Dimensionnement à fixer (masse, poussée, Isp) : chaque allumage doit rester court devant la période orbitale (environ 92 min), sinon l'approximation impulsionnelle du transfert de Hohmann ne tient plus. Ordre de grandeur : 150 kg et 20 N donnent environ 5 min par allumage.

### Logiciel de vol

- Machine à états : INHIBÉE → ARMÉE → PRÉCHAUFFAGE → ALLUMAGE → REFROIDISSEMENT → INHIBÉE. L'état de repos ne s'appelle pas SAFE, pour ne pas le confondre avec le mode SAFE du satellite.
- Commande critique en deux temps (armement puis exécution), avec inhibitions empêchant tout allumage hors séquence.
- Allumages déclenchés par commandes temporisées, souvent hors visibilité de la station sol.
- Surveillance pendant l'allumage (pression, températures, attitude), coupure par le FDIR en cas d'anomalie.
- Jaugeage de l'ergol par deux méthodes : comptage du temps d'allumage et méthode PVT (pression, volume, température). Les écarts sont comparés au sol.

### Pannes à injecter

- Propulseur bloqué ouvert : fermeture de la vanne d'isolement en amont.
- Propulseur qui ne s'allume pas.
- Fuite : la pression baisse alors que tout est fermé.
- Sous-performance.
- Réchauffeur en panne : risque de gel des lignes.

## Segment sol (`ground/`)

### Passerelle (`ground/gateway/`, Node + TypeScript)

- UDP ↔ WebSocket.
- Encodage et décodage du protocole sol-bord, d'après l'ICD.
- Archivage de la télémesure.
- La liaison radio passe par le simulateur, qui la coupe hors visibilité : la passerelle ne connaît pas l'orbite.
- Relais du canal « banc de test » vers le simulateur, sur une route WebSocket et un port UDP distincts de la liaison radio.

### Interface (`ground/web/`, React)

- Carte avec trace au sol et prochains passages.
- Tableaux de bord énergie, thermique et propulsion (ergol et Δv restants).
- Journal d'événements, complété à chaque passage par le vidage de la télémesure stockée à bord.
- Console de télécommandes avec suivi des acquittements, et armement/exécution pour les commandes critiques.
- Planification de manœuvres : calcul des allumages, envoi en commandes temporisées, bilan prévu contre réalisé.
- Panneau d'injection de pannes (banc de test).

## Questions ouvertes

À trancher en phase 1, chacune par un ADR dans `docs/decisions/`.

- **Sémantique du bus** : un message publié au cycle N est-il lu au cycle N (si le destinataire passe après l'émetteur) ou toujours au cycle N+1 ? La seconde option ne dépend pas de l'ordre d'appel, mais ajoute 100 ms partout. En particulier, le FDIR passe en dernier : sa coupure d'un allumage n'atteint la propulsion qu'au cycle suivant, sauf si l'étape 5 applique ses inhibitions aux commandes d'actionneurs du cycle en cours.
- **Emplacement de `Command` et des messages du bus** : le service TC/TM décode et aiguille les TC, donc il connaît les commandes de chaque sous-système, alors que `sgr-flight-core` ne doit pas dépendre de `sgr-flight`. Soit le cœur est générique sur le type de message (`Bus<M>`, `Subsystem<M>`), soit ces types sont définis dans le cœur ou dans `sgr-protocol`.
- **Des pilotes aux sous-systèmes** : comment une mesure acquise par un pilote parvient-elle aux sous-systèmes ? Soit le pilote la publie sur le bus interne, soit elle est rangée dans une table de données que le `Context` donne à lire. La même question se pose en sens inverse pour les commandes aux équipements.

## Feuille de route

0. **Mise en place** : dépôt, workspace Cargo, projets Node et React, ICD v0 (dont l'inventaire des capteurs et des actionneurs).
1. **Squelette de bout en bout** : simulateur et logiciel de vol minimaux en lockstep, liaison radio relayée par le simulateur, quelques valeurs affichées en direct sur l'interface.
2. **TC/TM et sûreté** : protocole complet (ICD v1), acquittements, stockage et vidage de la télémesure, paramètres, modes, FDIR, injection de pannes.
3. **Propulsion** : modèle physique, machine à états, manœuvres, scénario LEOP de bout en bout (detumbling encore fictif : attitude parfaite).
4. **Attitude** : contrôle d'attitude simplifié, detumbling réel.
5. **Matériel** : portage du logiciel de vol sur STM32 Nucleo (`no_std`, Embassy ou RTIC), simulateur sur PC relié en liaison série (hardware-in-the-loop).

Pistes pour la suite : vote à trois instances du logiciel de vol (à la manière de SpaceX), propulsion électrique, passage au PUS, capsule habitée réutilisant la même base.

## Références

- Architecture de logiciel de vol : NASA cFS (C), F´ du JPL (C++), sat-rs (Rust, prototype).
- Segment sol : OpenMCT (NASA), Yamcs.
- Simulateurs opérationnels : SIMULUS (ESA/ESOC), auquel le centre de contrôle se connecte comme à une station sol.
- Validation du simulateur : GMAT et 42 (NASA) comme références de calcul d'orbite.
- Normes : CCSDS 133.0-B (Space Packet Protocol) pour l'enveloppe ; ECSS-E-ST-70-41C (PUS-C) en cas d'évolution vers le PUS. Ses services 1 (vérification des requêtes), 11 (commandes temporisées) et 15 (stockage bord) inspirent déjà le protocole maison.
- Tests : la crate `spacepackets` peut servir d'oracle pour vérifier l'encodage de l'en-tête CCSDS.
