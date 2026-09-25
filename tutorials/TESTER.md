# Tester un jeu Bevy

Ce guide rassemble toutes les techniques de test utilisées dans les tutoriels. Gardez-le sous la
main quand vous écrivez les tests de vos propres jeux !

## Pourquoi tester un jeu ?

Un jeu change sans arrêt : on ajoute une règle, on accélère un ennemi, on réorganise le code...
et on casse souvent, sans s'en rendre compte, quelque chose qui fonctionnait. Rejouer tout le jeu
à la main après chaque modification est long et peu fiable. Des tests automatiques vérifient
tout en quelques secondes, à chaque fois.

Et Bevy s'y prête très bien : un jeu Bevy peut tourner **sans fenêtre**, image par image, sous le
contrôle complet du test.

## Lancer les tests

```sh
cargo test -p bevy_tutorials                              # les tests des solutions
cargo test -p bevy_tutorials --test exercices             # les tests de vos exercices
cargo test -p bevy_tutorials --test exercices chapitre_03 # un seul chapitre
cargo test -p bevy_tutorials --test exercices le_poison   # les tests dont le nom contient « le_poison »
cargo test -p bevy_tutorials -- --nocapture               # affiche aussi les `println!`
```

## L'anatomie d'un test

```rust
#[test]
fn le_compteur_augmente_a_chaque_image() {
    // 1. Préparer : une application avec seulement ce qu'on veut tester.
    let mut app = App::new();
    app.init_resource::<CompteurImages>();
    app.add_systems(Update, compter_les_images);

    // 2. Agir : exécuter des images.
    app.update();
    app.update();

    // 3. Vérifier : lire le résultat dans le monde.
    assert_eq!(app.world().resource::<CompteurImages>().0, 2);
}
```

- `App::new()` crée une application vide : pas de fenêtre, pas de rendu, pas de temps. C'est
  souvent suffisant.
- `bevy_tutorials::outils::app_de_test()` ajoute les `MinimalPlugins`, dont le temps (`Time`).
- `app.update()` exécute **une** image : la première exécute aussi les systèmes de démarrage
  (`Startup`).

## Préparer le monde

```rust
// Des ressources.
app.init_resource::<Score>(); // avec la valeur par défaut
app.insert_resource(Vie(3)); // avec une valeur choisie

// Des entités : `id()` renvoie leur identifiant, pour les retrouver ensuite.
let joueur = app.world_mut().spawn((Joueur, Transform::default())).id();

// Des systèmes, des plugins, des observateurs.
app.add_systems(Update, deplacer);
app.add_plugins(PluginPieces { valeur: 5 });
app.add_observer(subir_les_coups);
```

Pour tester un système **seul**, n'ajoutez que lui, avec les ressources et les entités dont il a
besoin : quand le test échoue, on sait tout de suite où chercher.

## Lire le monde

```rust
// Une ressource.
let score = app.world().resource::<Score>().0;

// Un composant d'une entité : `None` si l'entité n'a pas ce composant.
let vie = app.world().get::<Vie>(joueur);

// L'entité existe-t-elle encore ?
assert!(app.world().get_entity(joueur).is_err()); // elle a été détruite

// Toutes les entités qui ont certains composants.
let mut requete = app.world_mut().query_filtered::<&Transform, With<Piece>>();
for transform in requete.iter(app.world()) { ... }
let joueur = requete.single(app.world()).unwrap(); // exactement une entité

// Compter les entités qui ont un composant.
let nombre = bevy_tutorials::outils::compter::<Piece>(&mut app);
```

Attention : l'ordre dans lequel une requête parcourt les entités n'est pas garanti. Triez les
résultats avant de les comparer à une liste.

## Contrôler le temps

Dans un test, le temps ne doit pas avancer tout seul : le résultat dépendrait de la vitesse de
l'ordinateur. Avec `app_de_test()`, les outils suivants font avancer le temps d'une durée
précise :

```rust
use bevy_tutorials::outils::{image, simuler, proche};

image(&mut app, Duration::from_millis(250)); // une image de 250 ms
simuler(&mut app, 2.5); // 25 images de 100 ms

assert!(proche(position.x, 50.0)); // égal à un millième près
```

- Bevy limite la durée d'une image à 250 ms : pour simuler plus longtemps, utilisez plusieurs
  images (c'est ce que fait `simuler`).
- Ne comparez jamais des `f32` avec `==` après des calculs : les arrondis donnent des résultats
  comme `49.999996`. Utilisez `proche`.

## Simuler le clavier

```rust
app.init_resource::<ButtonInput<KeyCode>>(); // le clavier n'existe pas sans fenêtre

// Une touche maintenue enfoncée :
app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowRight);
simuler(&mut app, 1.0);
app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ArrowRight);

// Un appui bref, d'une image :
bevy_tutorials::outils::appuyer_sur(&mut app, KeyCode::Space);
```

Dans un vrai jeu, le `InputPlugin` remet à zéro les touches « tout juste appuyées »
(`just_pressed`) au début de chaque image. Sans lui, c'est au test de le faire, avec
`clear()` ; `appuyer_sur` s'en occupe.

## Les messages

```rust
app.add_message::<PieceRamassee>();

// Envoyer un message :
app.world_mut().write_message(PieceRamassee { valeur: 3 });

// Lire les messages envoyés pendant les deux dernières images :
let messages = app.world().resource::<Messages<PieceRamassee>>();
let envoyes: Vec<_> = messages.get_cursor().read(messages).collect();
```

## Les observateurs

```rust
app.add_observer(subir_les_coups);

// Déclencher un événement : les observateurs s'exécutent immédiatement.
app.world_mut().trigger(Touche { entity: ennemi, degats: 1 });

// Les commandes des observateurs (créer, détruire...) attendent le prochain `flush` :
app.world_mut().flush();
```

## Les états

```rust
use bevy::state::app::StatesPlugin;

app.add_plugins(StatesPlugin); // fait partie de `DefaultPlugins`, pas de `MinimalPlugins`

// Aller directement dans un état :
app.world_mut().resource_mut::<NextState<EtatJeu>>().set(EtatJeu::EnJeu);
app.update(); // le changement a lieu au début de cette image

// Lire l'état actuel :
let etat = *app.world().resource::<State<EtatJeu>>().get();
```

## Les paniques attendues

Pour vérifier qu'un code s'arrête avec une erreur :

```rust
#[test]
#[should_panic(expected = "plugin was already added")]
fn un_plugin_ne_peut_etre_ajoute_qu_une_fois() { ... }
```

## Ce qu'on ne teste pas sans fenêtre

Le rendu (les images, les couleurs, les animations à l'écran) ne se teste pas facilement. C'est
pourquoi il vaut mieux **séparer** les règles du jeu de son affichage, comme au chapitre 9 : les
règles sont entièrement testées, et l'affichage, simple, se vérifie en jouant.

Pour vérifier l'affichage automatiquement, Bevy peut prendre des captures d'écran pendant
l'exécution d'un exemple : voir la fonctionnalité `bevy_ci_testing` dans `docs/cargo_features.md`.

## Des tests qui aident

- **Un test, une idée** : son nom dit ce qu'il vérifie, en phrase
  (`le_poison_est_retire_quand_il_est_epuise`). Quand il échoue, on comprend tout de suite quoi.
- **Des messages clairs** : `assert!(proche(x, 50.0), "après 0,5 s : {x}")` affiche la valeur
  obtenue en cas d'échec.
- **Des tests rapides** : évitez les longues simulations, et préférez tester une fonction seule
  (comme `direction` au chapitre 5) quand c'est possible.
- **Écrire le test d'abord** : décrivez ce que la nouvelle fonctionnalité doit faire dans un
  test, vérifiez qu'il échoue, puis écrivez le code. C'est exactement ce que font les exercices !
