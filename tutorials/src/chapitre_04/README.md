# Chapitre 4 — Le temps : les minuteurs et les déplacements réguliers

Objectifs :

- déplacer des entités à la même vitesse, quelle que soit la puissance de l'ordinateur ;
- déclencher des actions à intervalles réguliers avec des **minuteurs** (`Timer`) ;
- utiliser `Transform`, la position des entités dans Bevy ;
- contrôler le temps dans les tests.

## Le problème des images

Au chapitre 2, les entités avançaient d'un pas **par image**. Mais un ordinateur rapide affiche
144 images par seconde, un plus lent 30 : le jeu irait presque 5 fois plus vite sur le premier !
La solution est de multiplier la vitesse par la **durée de l'image** :

```rust
fn deplacer(temps: Res<Time>, mut entites: Query<(&mut Transform, &Vitesse)>) {
    for (mut transform, vitesse) in &mut entites {
        transform.translation += (vitesse.0 * temps.delta_secs()).extend(0.0);
    }
}
```

Avec une vitesse de 100 pixels par seconde, une entité avance de 100 pixels en une seconde, que
cette seconde soit découpée en 10 images de 100 ms ou en 4 images de 250 ms.

La ressource `Time` donne :

| Méthode             | Valeur                                                  |
|---------------------|---------------------------------------------------------|
| `delta_secs()`      | la durée de l'image, en secondes (`f32`)                |
| `delta()`           | la même durée, en `Duration`                            |
| `elapsed_secs()`    | le temps écoulé depuis le lancement du jeu, en secondes |

## `Transform`

`Transform` est le composant de Bevy qui place les entités dans le monde : sa `translation`
(la position, un `Vec3`), sa `rotation` et son `scale` (la taille). C'est lui qu'utilisent le
rendu, la physique, le son spatialisé... En 2D, on ignore simplement l'axe `z` (ou on s'en sert
pour choisir quelles entités sont dessinées au-dessus des autres).

```rust
commands.spawn((Piece, Transform::from_xyz(100.0, 50.0, 0.0)));
```

## Les minuteurs

Un `Timer` compte le temps qui passe jusqu'à une durée donnée :

```rust
// Toutes les 2 secondes, indéfiniment.
let mut minuteur = Timer::from_seconds(2.0, TimerMode::Repeating);
// Une seule fois, au bout de 5 secondes.
let duree_de_vie = Timer::from_seconds(5.0, TimerMode::Once);
```

Il n'avance pas tout seul : il faut appeler `tick` à chaque image, avec la durée de l'image.

```rust
fn faire_apparaitre_des_pieces(
    mut commands: Commands,
    temps: Res<Time>,
    mut minuteur: ResMut<MinuteurPieces>,
) {
    minuteur.0.tick(temps.delta());
    if minuteur.0.just_finished() {
        commands.spawn(Piece);
    }
}
```

- `just_finished()` : le minuteur s'est terminé **pendant cette image** (une seule fois, même
  pour un minuteur qui recommence) ;
- `is_finished()` : le minuteur est terminé (un minuteur `Once` le reste ensuite).

Un minuteur peut être une ressource (un seul pour tout le jeu) ou un composant (un par entité,
comme `DureeDeVie`).

## Le temps dans les tests

Dans un vrai jeu, le temps avance tout seul. Dans un test, ce serait une catastrophe : le résultat
dépendrait de la vitesse de l'ordinateur ! Les outils des tutoriels contrôlent le temps :

```rust
use bevy_tutorials::outils::{app_de_test, image, proche, simuler};

let mut app = app_de_test(); // contient `Time`, grâce aux `MinimalPlugins`
app.add_systems(Update, deplacer);

simuler(&mut app, 0.5); // 5 images de 100 ms
image(&mut app, Duration::from_millis(250)); // une image de 250 ms
```

Bevy limite la durée d'une image à 250 ms (pour qu'un jeu qui a été mis en pause par le système
ne fasse pas un bond énorme) : au-delà, utilisez plusieurs images.

Les calculs sur les `f32` font des erreurs d'arrondi : on vérifie qu'un résultat est proche de la
valeur attendue avec `proche(x, 50.0)`, et pas qu'il est exactement égal.

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `MinuteurPieces::default` : un minuteur de 2 secondes qui recommence.
2. `deplacer` : selon la vitesse et le temps.
3. `faire_apparaitre_des_pieces` : une pièce toutes les 2 secondes, avec une durée de vie.
4. `faire_disparaitre` : détruit les entités dont la durée de vie est écoulée.
5. `mettre_a_jour_le_chrono` et `plugin`.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_04
```

## Pour aller plus loin

- Pour la physique, on préfère souvent un pas de temps **fixe** : la planification `FixedUpdate`
  s'exécute un nombre fixe de fois par seconde (64 par défaut). Voir
  `examples/movement/physics_in_fixed_timestep.rs`.
- `Time<Virtual>` peut être mis en pause ou accéléré (`set_relative_speed`) : pratique pour un
  ralenti ou un menu de pause.
