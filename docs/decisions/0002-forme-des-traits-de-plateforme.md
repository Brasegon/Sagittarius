# 0002 : forme des traits de plateforme

- **Date** : 2026-11-25
- **Statut** : acceptée

## Contexte

Le logiciel de vol tourne d'abord sur Linux, face au simulateur, puis sur une carte STM32 en phase 5. La couche plateforme isole ce qui change entre les deux : des traits définis dans `sgr-flight-core`, implémentés par une crate par plateforme.

Les deux premiers traits sont `Clock` et `EquipmentBus`. Leur forme engage tous les pilotes d'équipements et tous les services du cœur. Elle est donc fixée avant d'écrire le premier pilote, au moment où elle coûte le moins.

## Décision

1. **Les traits vivent dans `sgr_flight_core::platform`.** Les crates de plateforme les implémentent, le reste du logiciel de vol ne connaît qu'eux.
2. **`Clock` donne le temps bord en ticks** : `fn tick(&self) -> u32`, le numéro du cycle de 100 ms en cours, compté depuis la séparation du lanceur. C'est l'unité de l'ICD.
3. **`EquipmentBus` a une seule méthode, un échange demande-réponse bloquant** :

   ```rust
   fn transfer(
       &mut self,
       address: u8,
       request: &[u8],
       response: &mut [u8],
   ) -> Result<usize, BusError>;
   ```

   La réponse est copiée au début du buffer fourni par l'appelant, et la valeur renvoyée est sa longueur.
4. **Une réponse vide n'est pas un silence.** Un équipement qui a obéi sans rien avoir à dire donne `Ok(0)`. Un équipement muet donne `Err(BusError::NoResponse)`.
5. **`BusError` est un `enum` commun à toutes les plateformes**, avec deux variantes : `NoResponse` (le `NO_RESPONSE` de l'ICD, un délai dépassé sur un vrai bus) et `ResponseTooLong` (la réponse ne tient pas dans le buffer de l'appelant).
6. **Le trait ne porte pas le tick.** Il décrit un bus, pas le simulateur : c'est l'implémentation Linux qui ajoute le tick dans les trames lockstep.

## Conséquences

Ce qu'on y gagne :

- `sgr-flight-core` ne dépend ni de `sgr-protocol` ni de `heapless` pour ses traits. Le type `Payload` appartient au protocole lockstep, qui n'existe pas sur STM32.
- La forme est celle de `I2c::write_read` dans `embedded-hal`, le standard Rust des pilotes embarqués : le même trait pourra envelopper un vrai bus.
- Un pilote se teste avec un faux bus de quelques lignes, sans simulateur.
- Les pilotes ne sont pas génériques sur le type d'erreur : ils renvoient tous `BusError`.
- `&mut self` fait vérifier par le compilateur qu'il n'y a qu'un échange à la fois sur le bus.

Ce que ça coûte :

- L'appelant dimensionne son buffer, et doit lire la longueur renvoyée.
- Un `BusError` commun ne transporte pas le détail d'une erreur propre à une plateforme. On ajoutera une variante si l'implémentation Linux en révèle le besoin.
- L'appel est bloquant. Il convient à l'exécutif cyclique, mais Embassy, envisagé pour la phase 5, travaille en `async` : le trait sera à revoir à ce moment-là.

Les écarts avec un vrai satellite :

- Le temps bord y est daté en secondes et fractions de seconde depuis une époque de référence (format CUC, CCSDS 301.0-B), et recalé sur le sol ou sur le GPS. Ici, un compteur de cycles suffit.
- Un ordinateur de bord a plusieurs bus, de types différents. Ici, un seul trait couvre un bus générique (voir l'ICD, « Écarts avec un vrai satellite »).

Ce qui reste ouvert :

- **La cadence du cycle** : attendre le `Tick`, envoyer le `CycleDone`. Ce sera un troisième trait, à écrire avec la boucle principale.
- **`GroundLink`** : il attend l'ICD v0.3.
- **La dépendance de `sgr-flight` à `sgr-protocol`** : `crates/sgr-flight/Cargo.toml` l'interdit, alors que la décision 0001 range les formats des équipements dans `sgr-protocol`. À trancher avant le pilote du PCDU.
