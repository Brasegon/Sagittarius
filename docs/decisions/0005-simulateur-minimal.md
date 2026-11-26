# 0005 : forme du simulateur minimal

- **Date** : 2026-11-26
- **Statut** : acceptée

## Contexte

Le logiciel de vol sait jouer un cycle (décision 0004), mais seulement face à une fausse plateforme écrite dans ses tests. Pour le premier test de bout en bout, il lui faut le simulateur en face : un PCDU qui répond sur le bus, et une batterie dont la température évolue.

Les tests du logiciel de vol contiennent deux ébauches du modèle, `PcduModel` et `FakePlatform`. Elles mélangent l'équipement et ce qu'il mesure, et leur température ne bouge que si le test la change. Cela suffit pour tester un pilote, pas pour voir un thermostat réguler.

## Décision

1. **Trois pièces dans `sgr-sim`**, chacune dans son module :
   - `world::World` : la physique ;
   - `equipment::pcdu::Pcdu` : le modèle de l'équipement ;
   - `sim::Sim` : l'assemblage, derrière le bus d'équipements.
2. **Le monde est en unités physiques et en `f64`** : degrés Celsius, volts. Il ne connaît ni le bus ni les formats de l'ICD. Pour commencer, il a deux grandeurs : `bat_temp` et `bat_v`. La tension ne varie pas encore.
3. **La batterie est un nœud thermique unique.** Elle perd de la chaleur vers son environnement à travers une résistance thermique, et en reçoit de son réchauffeur quand il est alimenté. À chaque pas de 100 ms :

   ```
   perte     = (bat_temp − ENV_TEMP) / BAT_THERMAL_RESISTANCE
   apport    = BAT_HEATER_POWER si le réchauffeur est alimenté, 0 sinon
   bat_temp += (apport − perte) × TICK_SECONDS / BAT_HEAT_CAPACITY
   ```

   | Constante                | Valeur  | Sens                                                        |
   | ------------------------ | ------- | ----------------------------------------------------------- |
   | `TICK_SECONDS`           | 0,1 s   | Durée d'un tick                                             |
   | `ENV_TEMP`               | −10 °C  | Température de l'environnement de la batterie, cas froid    |
   | `BAT_HEAT_CAPACITY`      | 400 J/K | Capacité thermique de la batterie                           |
   | `BAT_THERMAL_RESISTANCE` | 5 K/W   | Résistance thermique entre la batterie et son environnement |
   | `BAT_HEATER_POWER`       | 6 W     | Puissance du réchauffeur                                    |

   Deux valeurs en découlent : une constante de temps de 2000 s (33 min), et un équilibre à +20 °C quand le réchauffeur reste alimenté.
4. **L'équipement mesure le monde et le traduit au format de l'ICD.** Le `Pcdu` reçoit le monde en lecture à chaque demande. Une mesure est arrondie au pas le plus proche, puis saturée aux bornes de son type.
5. **Le `Pcdu` garde l'état réel de ses interrupteurs.** `SET_SWITCHES` le remplace d'un coup, bits réservés ignorés. Une demande qu'il ne comprend pas reste sans réponse et ne change rien.
6. **Une réponse est un `Option<Payload>`** : `Some` porte les octets de la réponse, qui peut être vide, et `None` est un silence.
7. **`Sim` est l'interface en mémoire du cycle lockstep.** `tick` donne le tick en cours, `bus_request` aiguille une demande vers l'équipement dont c'est l'adresse, `step` avance le monde de 100 ms puis passe au tick suivant. Seul `step` change le monde : il est figé pendant le cycle.
8. **L'état de départ est donné par l'appelant** : `Sim::new(world)`. C'est le scénario qui fixe les conditions initiales.
9. **Le simulateur ne manipule pas encore les trames lockstep.** Le binaire UDP les ajoutera autour de cette interface.

## Conséquences

Ce qu'on y gagne :

- Le premier test de bout en bout montrera une boucle fermée : le thermostat du logiciel de vol qui régule une température simulée.
- Le bruit de mesure et les pannes d'équipement auront leur place : dans le modèle de l'équipement, pas dans la physique.
- Le monde se teste seul, sans bus ni octets. L'équipement se teste face à un monde figé.
- Le modèle est testé contre les trames de référence de l'ICD, écrites à la main dans ses tests : une erreur partagée avec le pilote ne passerait pas inaperçue.

Ce que ça coûte :

- Les valeurs du modèle thermique sont des ordres de grandeur pour un CubeSat 6U, choisis pour que les calculs tombent juste. Elles seront à revoir avec un vrai bilan thermique.
- `step` reçoit l'état du réchauffeur en argument. Avec d'autres actionneurs, il faudra regrouper ces entrées.
- Les champs `world` et `pcdu` de `Sim` sont publics. C'est pratique pour un scénario, mais rien n'empêche un test de modifier le monde en plein cycle.
- Les ébauches des tests du logiciel de vol restent en place : le test d'un pilote n'a pas besoin du simulateur.

Les écarts avec un vrai simulateur :

- Un modèle thermique réel compte des dizaines à des centaines de nœuds, et l'échange avec l'espace se fait par rayonnement, en T⁴. Ici, un seul nœud et une loi linéaire. Référence : Gilmore, *Spacecraft Thermal Control Handbook*.
- L'intégration est un schéma d'Euler explicite, le plus simple qui soit. Il suffit ici parce que le pas (0,1 s) est très petit devant la constante de temps (2000 s).
- Un vrai capteur ajoute du bruit, un biais et une dérive. Ici, la mesure est exacte à l'arrondi près.
- Les simulateurs opérationnels assemblent des modèles écrits selon un standard commun (SMP, ECSS-E-ST-40-07), pour les réutiliser d'une mission à l'autre.

Ce qui reste ouvert :

- **L'environnement thermique.** `ENV_TEMP` est constant. Il dépendra de l'orbite (éclipse, soleil) quand elle existera.
- **La tension de la batterie.** Elle est constante, en attendant le modèle d'énergie.
- **Les trames lockstep dans `sgr-e2e`.** Les échanges en mémoire peuvent passer par l'encodage des trames, ou l'éviter. À trancher avec le premier test de bout en bout.
- **Les pannes.** Un équipement muet ou une mesure fausse se régleront sur le modèle de l'équipement, par un canal à définir avec l'ICD v1.
