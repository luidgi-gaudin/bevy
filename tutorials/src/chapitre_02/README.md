# Chapitre 2 — Les entités, les composants et les requêtes

Objectifs :

- comprendre l'**ECS** (*Entity Component System*), le cœur de Bevy ;
- créer des **entités** faites de **composants** ;
- les retrouver et les modifier avec des **requêtes**, et les trier avec des **filtres**.

## Entités et composants

Dans Bevy, tout ce qui existe dans le jeu (le joueur, un ennemi, un arbre, la caméra...) est une
**entité**. Une entité n'est qu'un identifiant : ce sont ses **composants** qui lui donnent des
propriétés. Un composant est une structure Rust qui dérive `Component` :

```rust
#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct Vitesse(pub Vec2);

/// Un composant sans données sert d'étiquette : on l'appelle un « marqueur ».
#[derive(Component)]
pub struct Joueur;
```

Pour créer une entité, on donne tous ses composants dans un tuple à `commands.spawn` :

```rust
fn creer_le_monde(mut commands: Commands) {
    commands.spawn((Joueur, Position(Vec2::ZERO), Vitesse(Vec2::new(1.0, 0.0))));
    // Un arbre : une position, mais ni vitesse ni marqueur.
    commands.spawn(Position(Vec2::new(5.0, 5.0)));
}
```

Il n'y a pas de classe « joueur » ou « arbre » : un arbre est simplement une entité qui a une
position mais pas de vitesse. Pour qu'il se mette à bouger, il suffirait de lui ajouter une
`Vitesse` !

Les systèmes, eux, contiennent toute la logique du jeu, et s'appliquent à toutes les entités qui
ont les bons composants. C'est l'ECS : les **E**ntités, les **C**omposants (les données) et les
**S**ystèmes (la logique) sont séparés.

## Les requêtes

Un système accède aux composants avec une **requête**, `Query`. Elle donne toutes les entités qui
ont les composants demandés :

```rust
fn deplacer(mut entites: Query<(&mut Position, &Vitesse)>) {
    for (mut position, vitesse) in &mut entites {
        position.0 += vitesse.0;
    }
}
```

- `&Vitesse` : la vitesse est lue ;
- `&mut Position` : la position est modifiée, la requête doit alors être `mut`, et parcourue
  avec `&mut entites` ;
- une entité qui n'a pas de `Vitesse` (l'arbre) n'est pas dans cette requête.

## Les filtres

Le deuxième paramètre d'une requête est un **filtre** : il trie les entités sans lire leurs
composants.

```rust
// Les entités qui ont une position et le marqueur `Joueur`.
Query<&Position, With<Joueur>>
// Les entités qui ont une position mais pas de vitesse.
Query<&Position, Without<Vitesse>>
// Plusieurs filtres à la fois : un tuple.
Query<&Position, (Without<Vitesse>, Without<Ennemi>)>
// Juste compter, sans rien lire : `()` comme donnée.
Query<(), With<Ennemi>>
```

## Une seule entité : `Single`

Quand il n'y a qu'une seule entité à chercher, comme le joueur, `Single` est plus pratique
qu'une requête :

```rust
fn suivre_le_joueur(joueur: Single<&Position, With<Joueur>>) {
    let position = joueur.0; // pas de boucle : c'est directement le composant
}
```

S'il n'y a aucun joueur, ou s'il y en a plusieurs, Bevy **n'exécute pas** le système. Avec une
`Query`, on utiliserait `joueurs.single()`, qui renvoie une erreur dans ces cas.

## Lire les entités dans un test

Dans un test, on prépare le monde en créant les entités directement, puis on vérifie leurs
composants :

```rust
let entite = app.world_mut().spawn((Position(Vec2::ZERO), Vitesse(Vec2::X))).id();
app.update();
let position = app.world().get::<Position>(entite).unwrap();
```

Et pour parcourir toutes les entités, on crée une requête sur le monde :

```rust
let noms: Vec<String> = app
    .world_mut()
    .query::<&Nom>()
    .iter(app.world())
    .map(|nom| nom.0.clone())
    .collect();
```

`bevy_tutorials::outils::compter::<Ennemi>(&mut app)` compte les entités qui ont un composant.

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs). Les systèmes n'ont pas encore les bons paramètres : c'est
à vous d'ajouter les requêtes dont ils ont besoin.

1. `creer_le_monde` : le héros, deux ennemis et un arbre (voir la documentation de la fonction).
2. `deplacer` : ajoute la vitesse à la position.
3. `compter_les_ennemis` et `compter_les_decors` : avec les bons filtres.
4. `choisir_une_cible` : le nom de l'ennemi le plus proche du joueur.
5. `plugin` : assemble le tout.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_02
```

## Pour aller plus loin

- Deux requêtes d'un même système qui modifient le même composant sont refusées par Bevy
  (essayez !). Les filtres `Without` permettent de prouver qu'elles ne concernent pas les mêmes
  entités.
- Regardez l'exemple `examples/ecs/ecs_guide.rs` du dépôt Bevy : il présente l'ECS en détail.
