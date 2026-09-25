# Chapitre 1 — Premiers pas : l'application, les systèmes et les ressources

Objectifs :

- comprendre ce qu'est une application Bevy, et comment elle s'exécute image par image ;
- écrire des **systèmes**, les fonctions qui font vivre le jeu ;
- stocker des données globales dans des **ressources** ;
- lire un test Bevy.

## L'application

Tout jeu Bevy commence par une [`App`](https://docs.rs/bevy/latest/bevy/app/struct.App.html) :

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // la fenêtre, le rendu, le clavier, le son...
        .add_systems(Update, dire_bonjour)
        .run();
}

fn dire_bonjour() {
    info!("Bonjour !");
}
```

`App::run` lance la **boucle de jeu** : Bevy exécute une **image** (en anglais *frame*), puis une
autre, puis une autre... une soixantaine de fois par seconde. À chaque image, il exécute des
**planifications** (*schedules*), qui contiennent des systèmes :

| Planification | Quand ?                                   | Exemple d'utilisation                    |
|---------------|-------------------------------------------|------------------------------------------|
| `Startup`     | une seule fois, au début de la 1re image  | créer la caméra, charger le niveau       |
| `Update`      | à chaque image                            | déplacer le joueur, compter les points   |

## Les systèmes

Un **système** est une simple fonction Rust. Ses paramètres disent à Bevy de quelles données elle
a besoin, et Bevy les lui donne quand il l'exécute :

```rust
fn compter_les_images(mut compteur: ResMut<CompteurImages>) {
    compteur.0 += 1;
}
```

On l'ajoute à une planification avec `add_systems` :

```rust
app.add_systems(Update, compter_les_images);
```

Plusieurs systèmes peuvent être ajoutés d'un coup avec un tuple. Par défaut, Bevy les exécute dans
n'importe quel ordre (et même en parallèle !). `.chain()` impose de les exécuter dans l'ordre
du tuple :

```rust
app.add_systems(Update, (compter_les_images, annoncer_les_paliers).chain());
```

## Les ressources

Une **ressource** est une donnée unique et globale : le score, les réglages, le temps... Il suffit
de dériver `Resource` :

```rust
#[derive(Resource, Default)]
pub struct CompteurImages(pub u32);
```

Il faut l'ajouter à l'application avant de l'utiliser :

```rust
app.init_resource::<CompteurImages>(); // avec sa valeur par défaut (`Default`)
app.insert_resource(CompteurImages(9)); // avec une valeur précise
```

Un système y accède avec :

- `Res<CompteurImages>` pour la **lire** ;
- `ResMut<CompteurImages>` pour la **modifier**.

Bevy utilise ces types pour savoir quels systèmes peuvent s'exécuter en même temps : deux
systèmes qui lisent la même ressource peuvent tourner en parallèle, mais pas s'ils la modifient.

> Si un système demande une ressource qui n'a pas été ajoutée, Bevy s'arrête avec une erreur qui
> donne le nom de la ressource manquante.

## Les plugins

Un **plugin** regroupe tout ce qu'il faut pour une fonctionnalité : ressources, systèmes... La
façon la plus simple d'en écrire un est une fonction qui reçoit l'application :

```rust
pub fn plugin(app: &mut App) {
    app.init_resource::<CompteurImages>()
        .add_systems(Update, compter_les_images);
}

// Et dans `main` :
App::new().add_plugins((DefaultPlugins, plugin)).run();
```

`DefaultPlugins` est lui-même un ensemble de plugins : tout Bevy est construit ainsi.

## Lire un test Bevy

Voici un test de ce chapitre (dans `tests.rs`) :

```rust
#[test]
fn le_compteur_augmente_a_chaque_image() {
    // 1. Préparer : une application qui ne contient que ce qu'on veut tester.
    let mut app = App::new();
    app.init_resource::<CompteurImages>();
    app.add_systems(Update, compter_les_images);

    // 2. Exécuter : `update` exécute une seule image.
    for _ in 0..5 {
        app.update();
    }

    // 3. Vérifier : on lit le résultat dans le monde du jeu.
    assert_eq!(app.world().resource::<CompteurImages>().0, 5);
}
```

Pas besoin de fenêtre ni de `DefaultPlugins` : on n'ajoute que ce que l'on teste, et on avance
image par image avec `app.update()`. Le guide [`TESTER.md`](../../TESTER.md) détaille toutes les
techniques de test utilisées dans ces tutoriels.

## À vous de jouer

Ouvrez [`exercice.rs`](exercice.rs) et remplacez les `todo!()` :

1. `ecrire_bienvenue` : écrit `"Bienvenue dans Bevy !"` dans la ressource `Bienvenue`.
2. `compter_les_images` : ajoute 1 au compteur.
3. `annoncer_les_paliers` : toutes les 10 images, ajoute `"10 images"`, `"20 images"`... au
   `Journal`. Attention, rien ne doit être annoncé avant la première image (compteur à 0).
4. `plugin` : ajoute les trois ressources, `ecrire_bienvenue` au démarrage, et les deux autres
   systèmes à chaque image, dans le bon ordre.

Lancez les tests après chaque étape :

```sh
cargo test -p bevy_tutorials --test exercices chapitre_01
```

Les tests d'une étape passent dès qu'elle est terminée. Si vous êtes bloqué, la solution est dans
[`solution.rs`](solution.rs).

## Pour aller plus loin

- Ajoutez `info!("{} images", compteur.0);` dans un système, et lancez les tests avec
  `-- --nocapture` : que se passe-t-il ? (Indice : les journaux ont besoin du `LogPlugin`, qui fait
  partie de `DefaultPlugins`.)
- Retirez `.chain()` du plugin : les tests passent toujours, et pourtant les paliers peuvent être
  annoncés une image trop tard. Écrivez un test qui détecte ce problème.
