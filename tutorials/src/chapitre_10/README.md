# Chapitre 10 — Viser à la souris

> Ce chapitre ouvre la **piste FPS** des tutoriels : les chapitres 10 à 16 construisent, pièce par
> pièce, le cœur d'un FPS compétitif, jusqu'à une arène jouable. Ils supposent que les chapitres
> 1 à 9 sont acquis.

Objectifs :

- transformer les mouvements de la souris en rotation du regard ;
- régler la sensibilité comme les joueurs compétitifs (en centimètres par tour) ;
- éviter le piège classique qui rend la visée dépendante du nombre d'images par seconde.

## Pourquoi la visée est si importante

Dans un FPS compétitif, la souris est le lien direct entre la main du joueur et le jeu. Un joueur
entraîné sait exactement de combien bouger la main pour se retourner ou viser une tête : la visée
doit donc être **parfaitement prévisible**. Pas de lissage, pas d'accélération, et surtout le
même résultat à 60 ou à 360 images par seconde.

## Deux angles : le lacet et le tangage

Le regard est décrit par deux angles, qu'on garde dans un composant `Orientation` :

- le **lacet** (*yaw*) : tourner à gauche ou à droite, autour de l'axe vertical ;
- le **tangage** (*pitch*) : lever ou baisser les yeux.

On les combine en une rotation, en tournant d'abord autour de l'axe vertical (Y), puis de l'axe
horizontal (X) :

```rust
pub fn rotation(orientation: &Orientation) -> Quat {
    Quat::from_euler(EulerRot::YXZ, orientation.lacet, orientation.tangage, 0.0)
}
```

Dans Bevy, une caméra sans rotation regarde vers `-Z`, l'axe `Y` pointe vers le haut, et un
angle positif autour de `Y` tourne vers la gauche.

Le tangage est limité à 89° : au-delà de 90°, on regarderait derrière soi la tête à l'envers. Le
lacet, lui, peut tourner indéfiniment, mais on le ramène entre -π et π pour que les calculs
restent précis même après des heures de jeu.

## La souris : une distance, pas une vitesse

Le `InputPlugin` additionne tous les mouvements de la souris d'une image dans la ressource
`AccumulatedMouseMotion` :

```rust
fn viser(
    souris: Res<AccumulatedMouseMotion>,
    sensibilite: Res<Sensibilite>,
    mut orientations: Query<&mut Orientation, With<Joueur>>,
) {
    let angle_par_point = (DEGRES_PAR_POINT * sensibilite.0).to_radians();
    for mut orientation in &mut orientations {
        orientation.lacet -= souris.delta.x * angle_par_point;
        orientation.tangage -= souris.delta.y * angle_par_point;
        // + les limites
    }
}
```

**Le piège** : au chapitre 4, on multipliait les vitesses par `temps.delta_secs()`. Il ne faut
surtout pas le faire ici ! Le mouvement de la souris est déjà une **distance** parcourue pendant
l'image. Le multiplier par la durée de l'image rendrait la visée plus lente quand le jeu tourne
plus vite. Le test `la_visee_ne_depend_pas_du_nombre_d_images_par_seconde` vérifie que la même
distance de souris donne toujours le même angle.

## La sensibilité des joueurs compétitifs

La plupart des FPS reprennent la convention de Quake : un point de souris tourne le regard de
0,022° × la sensibilité. Mais une souris à 1600 DPI envoie deux fois plus de points qu'une souris
à 800 DPI pour le même geste : les joueurs comparent donc leurs réglages en **centimètres par
tour** (*cm/360*), la distance dont il faut déplacer la souris pour faire un tour complet :

```text
points par tour     = 360 / (0,022 × sensibilité)
centimètres par tour = points par tour / DPI × 2,54
```

À 800 DPI avec une sensibilité de 1, il faut 52 cm pour faire un tour. Un menu d'options qui
affiche cette valeur aide les joueurs à retrouver leurs réglages d'un jeu à l'autre.

## Dans le vrai jeu

Pour viser, il faut aussi capturer le curseur, pour qu'il ne sorte pas de la fenêtre. C'est le
rôle du composant `CursorOptions` de la fenêtre (voir l'exemple `arene_fps`) :

```rust
cursor.grab_mode = CursorGrabMode::Locked;
cursor.visible = false;
```

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `centimetres_par_tour` et `normaliser_angle` : deux fonctions de calcul.
2. `rotation` et `direction_du_regard`.
3. `viser` : sans multiplier par le temps !
4. `orienter` et `plugin`.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_10
```

## Pour aller plus loin

- En visée (chapitre 14), le champ de vision se resserre : pour que la visée garde la même
  « sensation », de nombreux jeux réduisent la sensibilité dans le même rapport que le champ de
  vision.
- La souris n'est pas le seul moyen de viser : `AccumulatedMouseMotion` a un équivalent pour les
  manettes, les axes des joysticks (`Gamepad`), qui eux doivent être multipliés par le temps, car
  ils donnent une vitesse de rotation !
