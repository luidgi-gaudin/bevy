# Chapitre 16 — Des bots pour s'entraîner

Objectifs :

- donner des adversaires au joueur, avant d'avoir un serveur et des amis ;
- programmer un comportement crédible : voir, réagir, viser, tirer, se déplacer ;
- régler la difficulté avec quelques nombres.

## Un bot est un combattant comme les autres

Un bot a exactement les mêmes composants qu'un joueur : `Physique`, `Orientation`, `Commandes`,
`CommandesDeTir`, `Arme`, `Hitboxes`, `Vie`... Il n'a simplement pas le marqueur `Joueur` : ses
commandes ne viennent pas du clavier et de la souris, mais du système `penser`.

C'est un grand avantage de l'ECS : le mouvement, les collisions, le tir et le match (chapitres 11
à 15) fonctionnent pour les bots **sans une ligne de code en plus**, et les bots ne peuvent pas
tricher : ils vont à la même vitesse, tirent à la même cadence et subissent les mêmes règles.

## Voir

Un bot ne doit pas voir à travers les murs. Pour chaque ennemi, on trace un rayon des yeux du bot
jusqu'au point qu'il viserait : si un obstacle le coupe, l'ennemi est caché.

```rust
pub fn ligne_de_vue(depuis: Vec3, vers: Vec3, obstacles: &[Aabb3d]) -> bool
```

Parmi les ennemis visibles, le bot choisit le plus proche.

## Réagir, viser, tirer

Un bot qui tirerait dès qu'un ennemi apparaît serait injouable. Trois réglages le rendent humain :

| Réglage               | Facile | Difficile | Effet                                           |
|-----------------------|--------|-----------|-------------------------------------------------|
| `temps_de_reaction`   | 0,6 s  | 0,25 s    | délai avant le premier tir sur une nouvelle cible |
| `vitesse_de_rotation` | 90°/s  | 360°/s    | vitesse à laquelle il tourne la tête            |
| `tolerance`           | 4°     | 1°        | écart de visée maximal pour tirer               |
| `hauteur_visee`       | torse  | tête      | où il vise                                      |

Le bot tourne son regard vers sa cible pas à pas, comme un joueur qui bouge sa souris, avec
`tourner_vers` : sans dépasser sa vitesse de rotation, et par le chemin le plus court (de 170° à
-170°, il tourne de 20°, pas de 340°).

## Se déplacer

En combattant, le bot se décale de gauche à droite, pour être plus difficile à toucher (une
technique que les joueurs appellent le *strafe*). Sans ennemi en vue, il patrouille entre les
points d'apparition de la carte.

## Déterministe, donc testable

Le comportement des bots n'utilise aucun hasard : pour une même situation, un bot fait toujours
la même chose. Les tests peuvent donc vérifier qu'un bot ne voit pas à travers un mur, qu'il
attend son temps de réaction, ou qu'un bot difficile élimine une cible immobile en moins de deux
secondes.

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `orientation_vers`, `tourner_vers` et `ligne_de_vue` : la géométrie.
2. `penser` : le cerveau (suivez les étapes de sa documentation).
3. `plugin`.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_16
```

## Pour aller plus loin

- Les bots se souviennent de la dernière position de leur cible : quand elle se cache, ils vont
  vérifier où ils l'ont vue pour la dernière fois.
- Un peu d'imprécision rend les bots plus naturels : ajoutez un petit décalage à leur visée, qui
  diminue pendant qu'ils suivent leur cible (en utilisant le hasard reproductible du chapitre 14).
