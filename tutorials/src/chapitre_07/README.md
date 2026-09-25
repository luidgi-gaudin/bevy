# Chapitre 7 — Les états du jeu

Objectifs :

- découper le jeu en **états** : menu, partie, fin de partie ;
- exécuter des systèmes en entrant dans un état, ou seulement pendant un état ;
- nettoyer automatiquement les entités d'un état ;
- tester les changements d'état.

## Déclarer des états

Un jeu passe par plusieurs phases : l'écran titre, la partie, l'écran de fin... Dans Bevy, ce sont
des **états**, décrits par une énumération :

```rust
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EtatJeu {
    #[default]
    Menu,
    EnJeu,
    FinDePartie,
}

app.init_state::<EtatJeu>(); // commence dans l'état par défaut : `Menu`
```

Les états ont besoin du `StatesPlugin`, qui fait partie de `DefaultPlugins` (dans les tests, il
faut l'ajouter soi-même).

## Changer d'état

On lit l'état actuel avec `Res<State<EtatJeu>>`, et on demande à en changer avec
`ResMut<NextState<EtatJeu>>` :

```rust
fn lancer_la_partie(
    clavier: Res<ButtonInput<KeyCode>>,
    mut prochain_etat: ResMut<NextState<EtatJeu>>,
) {
    if clavier.just_pressed(KeyCode::Enter) {
        prochain_etat.set(EtatJeu::EnJeu);
    }
}
```

Le changement n'est pas immédiat : il a lieu **au début de l'image suivante**, dans la
planification `StateTransition`, qui s'exécute avant `Update`.

## Des systèmes liés aux états

| Planification ou condition              | Le système s'exécute...                          |
|-----------------------------------------|--------------------------------------------------|
| `OnEnter(EtatJeu::EnJeu)`               | une fois, en entrant dans l'état                 |
| `OnExit(EtatJeu::EnJeu)`                | une fois, en quittant l'état                     |
| `systeme.run_if(in_state(EtatJeu::EnJeu))` | à chaque image, mais seulement dans cet état  |

```rust
app.add_systems(OnEnter(EtatJeu::EnJeu), preparer_la_partie)
    .add_systems(Update, decompter_le_temps.run_if(in_state(EtatJeu::EnJeu)));
```

`run_if` accepte d'autres **conditions** : `resource_exists::<R>`, `on_timer(duree)`... et même
vos propres fonctions qui renvoient un `bool`.

## Nettoyer en quittant un état

À la fin d'une partie, le joueur, les ennemis, les pièces... doivent disparaître. Plutôt que de
les détruire un par un dans `OnExit`, on les marque dès leur création :

```rust
commands.spawn((Joueur, DespawnOnExit(EtatJeu::EnJeu)));
```

Bevy détruit automatiquement toutes les entités marquées quand on quitte l'état.

## Les états dans les tests

```rust
use bevy::state::app::StatesPlugin;

let mut app = app_de_test();
app.add_plugins(StatesPlugin);
app.add_plugins(plugin);

// Aller directement dans un état, sans passer par les menus :
app.world_mut().resource_mut::<NextState<EtatJeu>>().set(EtatJeu::EnJeu);
app.update();

// Lire l'état actuel :
assert_eq!(*app.world().resource::<State<EtatJeu>>().get(), EtatJeu::EnJeu);
```

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `lancer_la_partie` et `revenir_au_menu` : changent d'état avec la touche Entrée.
2. `preparer_la_partie` : le joueur (qui disparaîtra à la fin de la partie), le temps restant et le
   nombre de parties.
3. `decompter_le_temps` : termine la partie quand le temps est écoulé.
4. `plugin` : l'état et chaque système au bon moment.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_07
```

## Pour aller plus loin

- Ajoutez un état `Pause`, activé et désactivé par la touche Échap. Attention : quitter `EnJeu`
  pour `Pause` détruirait le joueur ! Les **sous-états** (`SubStates`) résolvent ce problème :
  voir `examples/state/sub_states.rs`.
- Les **états calculés** (`ComputedStates`) sont déduits d'autres états : voir
  `examples/state/computed_states.rs`.
