# Chapitre 5 — Les entrées du clavier

Objectifs :

- lire l'état du clavier avec `ButtonInput<KeyCode>` ;
- distinguer une touche **enfoncée** d'une touche **tout juste appuyée** ;
- isoler la logique dans des fonctions ordinaires, faciles à tester ;
- simuler le clavier dans les tests.

## `ButtonInput<KeyCode>`

Le `InputPlugin` (inclus dans `DefaultPlugins`) tient à jour la ressource
`ButtonInput<KeyCode>`, qui donne l'état de chaque touche :

| Méthode                    | Vrai...                                                       |
|----------------------------|---------------------------------------------------------------|
| `pressed(touche)`          | tant que la touche est enfoncée : pour se déplacer, viser...  |
| `just_pressed(touche)`     | seulement pendant l'image où la touche a été enfoncée : sauter, tirer, ouvrir un menu |
| `just_released(touche)`    | seulement pendant l'image où la touche a été relâchée         |
| `any_pressed([a, b])`      | si au moins une des touches est enfoncée                      |

```rust
fn tirer(mut commands: Commands, clavier: Res<ButtonInput<KeyCode>>) {
    if clavier.just_pressed(KeyCode::Space) {
        commands.spawn(Projectile);
    }
}
```

Avec `pressed`, garder la barre d'espace enfoncée tirerait un projectile à chaque image, soit
60 par seconde !

La souris fonctionne de la même façon avec `ButtonInput<MouseButton>`, et les manettes avec
`Gamepad` (voir `examples/input/`).

## AZERTY ou QWERTY ?

`KeyCode` désigne l'**emplacement** d'une touche sur le clavier, et pas la lettre qui y est
imprimée. `KeyCode::KeyW` est la touche en haut à gauche des lettres : W sur un clavier QWERTY,
mais Z sur un clavier AZERTY. Ainsi, les commandes « WASD » deviennent automatiquement « ZQSD »
pour les joueurs français. (Pour du texte, comme un nom de joueur, il faut au contraire lire les
lettres tapées : voir `examples/input/char_input_events.rs`.)

## Séparer la logique

La direction choisie au clavier est calculée par une fonction ordinaire, qui ne dépend pas de Bevy
autrement que par le clavier qu'elle reçoit :

```rust
pub fn direction(clavier: &ButtonInput<KeyCode>) -> Vec2 { ... }

fn deplacer_le_joueur(
    clavier: Res<ButtonInput<KeyCode>>,
    temps: Res<Time>,
    mut joueur: Single<&mut Transform, With<Joueur>>,
) {
    let deplacement = direction(&clavier) * VITESSE_JOUEUR * temps.delta_secs();
    joueur.translation += deplacement.extend(0.0);
}
```

Cette fonction se teste sans application, en quelques lignes : c'est souvent la façon la plus
simple de tester un calcul compliqué.

```rust
let mut clavier = ButtonInput::<KeyCode>::default();
clavier.press(KeyCode::ArrowUp);
assert_eq!(direction(&clavier), Vec2::Y);
```

## Simuler le clavier dans un test

Sans fenêtre, pas de vrai clavier : le test ajoute la ressource lui-même, puis appuie sur les
touches.

```rust
app.init_resource::<ButtonInput<KeyCode>>();
app.world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
simuler(&mut app, 0.5);
```

Attention : c'est le `InputPlugin` qui remet à zéro les touches « tout juste appuyées » au début
de chaque image. Sans lui, une touche reste `just_pressed` jusqu'à ce que le test appelle
`clear()`. L'outil `appuyer_sur(&mut app, KeyCode::Space)` s'en occupe : il simule un appui bref,
d'une image.

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `direction` : la direction choisie avec les flèches ou WASD/ZQSD, de longueur 1.
2. `deplacer_le_joueur` : utilise `direction`, et court avec Maj.
3. `tirer` : un seul projectile par appui sur la barre d'espace.
4. `plugin` : le joueur au démarrage, et les deux systèmes.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_05
```

## Pour aller plus loin

- Les tests `les_fleches_donnent_la_direction` et `les_touches_wasd_fonctionnent_aussi` sont
  très répétitifs. Réécrivez-les avec une boucle sur un tableau de cas `(touche, direction)`.
- Plutôt que des touches fixes, un jeu peut laisser les joueurs choisir leurs commandes : stockez
  les touches dans une ressource `Commandes`, lue par `direction`.
