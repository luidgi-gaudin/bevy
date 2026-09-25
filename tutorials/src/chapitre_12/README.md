# Chapitre 12 — Les collisions avec la carte

Objectifs :

- empêcher le joueur de traverser les murs ;
- le laisser glisser le long des murs, et monter sur les caisses ;
- déjouer les pièges des nombres à virgule.

## La carte en boîtes

Une carte de FPS compétitif est faite de murs, de caisses et de plateformes. Pour les collisions,
on les simplifie en **boîtes alignées sur les axes** (en anglais *AABB*, pour *axis-aligned
bounding box*), qu'on décrit par deux coins, le minimum et le maximum :

```rust
#[derive(Component)]
pub struct Obstacle(pub Aabb3d);

commands.spawn(Obstacle::new(Vec3::new(0.0, 0.5, -3.0), Vec3::new(4.0, 1.0, 2.0)));
```

Le joueur est lui aussi une boîte, de 0,8 m de large et 1,8 m de haut, posée sur ses pieds.
`Aabb3d` vient de `bevy::math` : Bevy fournit aussi des sphères, des capsules, des rayons, et les
tests d'intersection entre eux.

> Les jeux professionnels utilisent des formes plus précises (des capsules, ou directement les
> triangles de la carte) et un moteur physique, comme les bibliothèques Avian ou Rapier pour
> Bevy. Mais le principe reste le même, et les boîtes suffisent pour un premier jeu.

## Se déplacer axe par axe

Le tick de mouvement du chapitre 11 calcule où le joueur **voudrait** aller. On corrige ensuite
ce déplacement, un axe à la fois : X, puis Z, puis Y. Sur chaque axe, si la boîte du joueur
chevauche un obstacle, on la recule jusqu'au contact, et on annule la vitesse sur cet axe.

Traiter les axes séparément donne gratuitement un comportement essentiel : **glisser le long des
murs**. Un joueur qui court en diagonale contre un mur est arrêté sur un axe, mais continue sur
l'autre.

Et l'axe Y donne le sol : un joueur arrêté en descendant s'est posé sur une caisse. Comme la
gravité s'applique à chaque tick, même au sol (chapitre 11), le joueur s'enfonce très légèrement
à chaque tick, et la collision le repose aussitôt sur la caisse : il sait ainsi qu'il est au sol.

```rust
app.add_systems(FixedUpdate, resoudre_les_collisions.after(simuler_le_mouvement));
```

## Le piège des arrondis

Après une collision, le joueur est placé **exactement** au contact du mur. Mais avec les nombres
à virgule, « exactement » peut vouloir dire un milliardième de mètre trop loin, dans le mur. Au
tick suivant, s'il longe le mur, on croirait qu'il le traverse, et il se bloquerait sans raison.

La solution : deux boîtes ne sont en collision que si elles se chevauchent de plus d'une petite
**marge** (un dixième de millimètre). Le test `toucher_un_mur_n_empeche_pas_de_le_longer`
vérifie précisément ce cas.

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `boite_du_joueur` et `se_chevauchent`.
2. `deplacer_avec_collisions` : le cœur du chapitre.
3. `corriger`, `resoudre_les_collisions` et `plugin`.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_12
```

Le test `en_sautant_on_monte_sur_la_caisse` combine les chapitres 11 et 12 : un saut en courant
permet de monter sur une caisse d'un mètre.

## Pour aller plus loin

- L'effet tunnel : un objet très rapide peut traverser un obstacle fin en un seul tick, si sa
  boîte passe entièrement de l'autre côté. Pour le joueur, haut de 1,8 m, il faudrait tomber à
  plus de 240 m/s : aucun risque. Mais une grenade de 10 cm lancée à 30 m/s traverse-t-elle un mur
  de 10 cm ? Calculez-le, puis écrivez le test !
- Les escaliers : dans la plupart des FPS, le joueur monte automatiquement les petites marches
  (*step up*). Essayez : si un mouvement horizontal est bloqué, tentez-le 30 cm plus haut.
