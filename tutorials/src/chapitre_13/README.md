# Chapitre 13 — Les armes hitscan

Objectifs :

- tirer des balles instantanées (*hitscan*) avec des rayons ;
- découper les personnages en **hitbox** : la tête, le torse, les jambes ;
- calculer les dégâts selon la zone touchée et la distance ;
- gérer la cadence de tir, le chargeur et le rechargement, sans tricherie possible.

## Hitscan ou projectiles ?

Dans la plupart des FPS compétitifs, les balles des fusils sont **instantanées** : au moment du
tir, on trace un rayon depuis les yeux du tireur, dans la direction du regard, et on regarde ce
qu'il touche en premier. C'est le *hitscan*. Les roquettes et les grenades, plus lentes, sont de
vrais projectiles qui se déplacent tick après tick.

```rust
let rayon = RayCast3d::new(origine, direction, PORTEE_MAX);
if let Some(distance) = rayon.aabb_intersection_at(&boite) {
    // Le rayon touche la boîte à cette distance.
}
```

Le premier impact est le plus proche, parmi les murs de la carte (chapitre 12) et les hitbox des
autres personnages : un mur protège ceux qui sont derrière.

## Les hitbox

Tester chaque triangle du modèle 3D d'un personnage serait bien trop lent (et injuste : une
animation pourrait cacher la tête). Les jeux découpent donc chaque personnage en quelques boîtes
simples, les **hitbox**, chacune avec son multiplicateur de dégâts :

| Zone   | Hauteur       | Multiplicateur |
|--------|---------------|----------------|
| Tête   | 1,5 à 1,8 m   | × 2            |
| Torse  | 0,9 à 1,5 m   | × 1            |
| Jambes | 0 à 0,9 m     | × 0,8          |

Les tirs partent des yeux, à 1,6 m : un joueur qui vise droit devant lui touche la tête d'un
personnage de sa taille. Viser la tête est donc payant... mais c'est aussi la plus petite cible !

## Les dégâts

Un fusil d'assaut fait 30 dégâts par balle : il faut 4 balles dans le torse pour éliminer un
joueur de 100 points de vie, mais 2 dans la tête. À 600 tirs par minute, cela donne un *time to
kill* (le temps pour éliminer un adversaire) de 0,3 seconde, typique de Call of Duty.

Au-delà de sa **portée efficace**, une arme perd des dégâts : c'est ce qui distingue un fusil à
pompe d'un fusil de précision.

## La gâchette

`actionner_l_arme` fait avancer l'état de l'arme d'un tick. Deux subtilités :

1. **La cadence exacte.** À 600 tirs par minute, il faut 0,1 seconde entre deux balles, soit
   12,8 ticks. Si on attendait simplement 13 ticks, l'arme tirerait à 590 coups par minute. On
   reporte donc le temps en trop sur la balle suivante : en moyenne, la cadence est exacte.
2. **Pas de rafale accumulée.** Ce report ne doit valoir que gâchette enfoncée : sinon, un joueur
   qui attend (ou qui recharge) accumulerait du temps et tirerait ensuite plusieurs balles d'un
   coup. Le test `un_chargeur_vide_se_recharge_automatiquement` vérifie qu'aucune rafale ne suit
   un rechargement.

> Le tir a lieu dans `FixedUpdate`, à chaque tick, comme le mouvement : dans un jeu en réseau,
> c'est le serveur qui décide si une balle touche, avec exactement le même code.

## Les messages de tir

Chaque balle envoie un message `Tir` (pour afficher un éclair, jouer un son, ou calculer le recul
au chapitre 14), et chaque balle qui touche envoie un message `Touche` (pour retirer des points de
vie au chapitre 15, ou afficher un *hitmarker*).

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `Zone::multiplicateur` et `calculer_les_degats`.
2. `premier_impact` : le rayon contre les murs et les hitbox.
3. `actionner_l_arme` : la cadence, le chargeur et le rechargement.
4. `lire_les_commandes_de_tir`, `tirer` et `plugin`.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_13
```

## Pour aller plus loin

- Ajoutez d'autres armes : un fusil de précision (100 dégâts, 40 tirs par minute), un pistolet
  mitrailleur (20 dégâts, 900 tirs par minute, courte portée). Quel est le *time to kill* de
  chacune ?
- Certaines balles traversent les murs fins en perdant des dégâts (*wallbang*) : au lieu de
  s'arrêter au premier obstacle, continuez le rayon en réduisant les dégâts à chaque mur.
