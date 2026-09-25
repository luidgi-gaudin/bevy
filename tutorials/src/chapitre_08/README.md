# Chapitre 8 — Organiser le jeu : plugins et ordre des systèmes

Objectifs :

- découper un jeu en **plugins** indépendants, et les configurer ;
- regrouper les systèmes en **étapes** (`SystemSet`) et les ordonner ;
- tester chaque plugin séparément, puis le jeu complet.

## Les plugins, en version complète

Au chapitre 1, un plugin était une simple fonction. Quand il a besoin de réglages, on écrit une
structure qui implémente le trait `Plugin` :

```rust
pub struct PluginPieces {
    pub valeur: u32,
}

impl Plugin for PluginPieces {
    fn build(&self, app: &mut App) {
        app.insert_resource(ValeurPiece(self.valeur))
            .add_systems(Update, (ramasser_les_pieces, compter_les_points).chain());
    }
}

app.add_plugins(PluginPieces { valeur: 5 });
```

Un plugin peut en ajouter d'autres : c'est ainsi que `DefaultPlugins` regroupe des dizaines de
plugins. Un même plugin ne peut être ajouté qu'une fois : Bevy s'arrête avec une erreur sinon.

Un bon découpage suit les fonctionnalités du jeu : le joueur, les ennemis, les pièces,
l'interface, le son... Chaque plugin a ses propres composants, ressources et systèmes, et se
teste seul.

## Les étapes (`SystemSet`)

Par défaut, Bevy exécute les systèmes dans l'ordre qui lui convient, et en parallèle quand c'est
possible. C'est très efficace, mais le résultat peut dépendre de cet ordre : si les collisions
sont testées avant le déplacement du joueur, une pièce touchée n'est ramassée qu'à l'image
suivante.

Plutôt que d'ordonner chaque système par rapport aux autres, on les range dans des **étapes** :

```rust
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Etape {
    Entrees,
    Deplacement,
    Collisions,
}

// Chaque plugin range ses systèmes dans une étape...
app.add_systems(Update, deplacer_le_joueur.in_set(Etape::Deplacement));

// ... et le jeu ordonne les étapes, une fois pour toutes.
app.configure_sets(Update, (Etape::Entrees, Etape::Deplacement, Etape::Collisions).chain());
```

Les systèmes d'une même étape peuvent toujours s'exécuter en parallèle entre eux. Pour ordonner
deux systèmes précis, il existe aussi `.before(...)` et `.after(...)`, qui acceptent un système
ou une étape.

## Tester des plugins

Chaque plugin se teste seul, en préparant ce dont il a besoin :

```rust
let mut app = app_de_test();
app.init_resource::<ButtonInput<KeyCode>>();
app.world_mut().spawn((Joueur, Transform::default()));
app.add_plugins(PluginPieces { valeur: 5 });
```

Puis un test du jeu complet vérifie qu'ils fonctionnent ensemble : c'est un **test
d'intégration**. Ici, `les_etapes_s_executent_dans_l_ordre` vérifie qu'en **une seule image** le
clavier est lu, le joueur avance, et la pièce est ramassée.

Pour vérifier qu'un code s'arrête avec une erreur, un test peut attendre une panique :

```rust
#[test]
#[should_panic(expected = "plugin was already added")]
fn un_plugin_ne_peut_etre_ajoute_qu_une_fois() { ... }
```

## À vous de jouer

Dans [`exercice.rs`](exercice.rs), les systèmes sont déjà écrits. Complétez les trois plugins :

1. `PluginJoueur` : la ressource `Direction` et les deux systèmes, chacun dans son étape.
2. `PluginPieces` : le message, les ressources, et les deux systèmes dans l'étape `Collisions`.
3. `PluginJeu` : l'ordre des étapes, et les deux autres plugins.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_08
```

## Pour aller plus loin

- Bevy peut signaler les systèmes dont l'ordre n'est pas défini alors qu'ils modifient les mêmes
  données (on parle d'« ambiguïtés ») : voir `examples/ecs/nondeterministic_system_order.rs`.
- Les étapes peuvent aussi être désactivées d'un coup avec une condition :
  `app.configure_sets(Update, Etape::Deplacement.run_if(in_state(EtatJeu::EnJeu)))`.
