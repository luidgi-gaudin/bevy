# Chapitre 15 — Le match à mort

Objectifs :

- transformer les balles en dégâts, et les dégâts en éliminations ;
- régénérer la vie, comme dans Call of Duty ;
- faire réapparaître les combattants au bon endroit ;
- tenir les scores et désigner un vainqueur.

## Des balles aux éliminations

Au chapitre 13, chaque balle qui touche envoie un message `Touche`. Il reste à en tirer les
conséquences, en deux temps :

1. `infliger_les_degats` retire les dégâts aux points de vie de la cible. À zéro, il envoie une
   `Elimination`.
2. `enregistrer_les_eliminations` met à jour les statistiques, le fil des éliminations, et met la
   victime hors jeu.

Un piège : à 600 tirs par minute, avec plusieurs adversaires, deux balles peuvent achever la même
cible pendant le **même tick**. Sans précaution, elle serait éliminée deux fois ! On ignore donc
les balles qui touchent une cible déjà à zéro.

## Hors jeu, sans rien détruire

Un combattant éliminé ne doit plus bouger, tirer, ni être touché, pendant trois secondes. Plutôt
que de détruire son entité (et de perdre ses statistiques), on lui **retire des composants** :

```rust
commands
    .entity(victime)
    .insert(Mort { reapparition: DELAI_DE_REAPPARITION })
    .remove::<(Hitboxes, Commandes, CommandesDeTir)>();
```

Sans `Commandes`, les systèmes de mouvement (chapitre 11) et de tir (chapitres 13 et 14) ne le
trouvent plus dans leurs requêtes ; sans `Hitboxes`, les balles le traversent. C'est toute la
puissance de l'ECS : on change le comportement d'une entité en changeant ses composants, sans
toucher aux systèmes. À la réapparition, on lui rend ses composants.

## La régénération

Dans Call of Duty, la vie remonte toute seule après quelques secondes sans être touché : cela
récompense les joueurs qui se mettent à couvert. Chaque combattant garde le temps écoulé depuis
ses derniers dégâts ; au-delà de 4 secondes, il récupère 40 points de vie par seconde.

## Où réapparaître ?

Réapparaître devant un ennemi, c'est mourir aussitôt (*spawn kill*). On choisit donc, parmi les
points d'apparition de la carte, celui dont l'ennemi **le plus proche** est **le plus loin** :

```rust
pub fn choisir_un_point_d_apparition(points: &[Vec3], ennemis: &[Vec3]) -> Option<Vec3>
```

C'est une fonction ordinaire, testée sans application.

## `FixedPostUpdate`

Les systèmes du match s'exécutent dans `FixedPostUpdate`, qui suit `FixedUpdate` à chaque tick :
ils voient toujours les balles du tick en cours, que le tir vienne du chapitre 13 ou du
chapitre 14, sans avoir à connaître l'un ou l'autre. Et une condition les arrête quand le match
est terminé :

```rust
app.add_systems(FixedPostUpdate, (infliger_les_degats, ...).chain().run_if(match_en_cours));
```

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `match_en_cours` et `choisir_un_point_d_apparition`.
2. `infliger_les_degats` et `enregistrer_les_eliminations`.
3. `regenerer` et `faire_reapparaitre`.
4. `plugin`.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_15
```

Le test `un_duel_de_bout_en_bout` réunit les chapitres 11 à 15 : un joueur épaule, tire deux
balles dans la tête d'un adversaire à 5 mètres, puis l'adversaire réapparaît loin de lui.

## Pour aller plus loin

- Les séries d'éliminations (*killstreaks*) de Call of Duty : comptez les éliminations sans mourir,
  et envoyez un message à 3, 5 et 7 éliminations.
- Le match à mort par équipes : ajoutez un composant `Equipe`, ignorez les dégâts entre
  coéquipiers, et comptez les points par équipe.
