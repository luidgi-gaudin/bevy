# Chapitre 3 — Les commandes : créer, modifier et détruire des entités

Objectifs :

- créer et détruire des entités pendant la partie ;
- ajouter et retirer des composants ;
- comprendre quand les commandes sont appliquées ;
- utiliser les **composants requis**.

## Les commandes

Un système ne peut pas créer ou détruire une entité directement pendant qu'il parcourt une
requête : il demande à Bevy de le faire, avec des **commandes** (`Commands`).

```rust
fn retirer_les_morts(mut commands: Commands, entites: Query<(Entity, &Vie)>) {
    for (entite, vie) in &entites {
        if vie.0 == 0 {
            commands.entity(entite).despawn();
        }
    }
}
```

`Entity` est l'identifiant d'une entité : on l'ajoute à une requête pour savoir à quelle entité
appartiennent les composants. Les principales commandes sont :

| Commande                                  | Effet                                   |
|-------------------------------------------|-----------------------------------------|
| `commands.spawn((A, B))`                  | crée une entité avec les composants A, B |
| `commands.entity(e).despawn()`            | détruit l'entité `e`                    |
| `commands.entity(e).insert(C)`            | ajoute (ou remplace) le composant C     |
| `commands.entity(e).remove::<C>()`        | retire le composant C                   |

## Quand les commandes sont-elles appliquées ?

Les commandes sont **mises en attente**, puis appliquées plus tard, quand plus aucun système ne
lit le monde : à la fin de la planification, ou entre deux systèmes enchaînés avec `.chain()`.
Ainsi, dans :

```rust
app.add_systems(Update, (appliquer_le_poison, enrager_les_blesses, retirer_les_morts).chain());
```

- `appliquer_le_poison` modifie la `Vie` directement (avec `&mut Vie`) : `retirer_les_morts` voit
  la nouvelle valeur dans la même image ;
- le marqueur `Enrage` ajouté par `enrager_les_blesses` avec une commande existe dès le système
  suivant, parce que `.chain()` applique les commandes entre les systèmes.

> Une commande qui vise une entité qui n'existe plus échoue : pour les cas où c'est normal,
> `commands.get_entity(entite)` renvoie une erreur au lieu de laisser la commande échouer.

## Les composants requis

Certains composants n'ont de sens qu'avec d'autres : un slime est toujours un ennemi, et a
toujours des points de vie. Au lieu de penser à les ajouter à chaque fois, on les **requiert** :

```rust
#[derive(Component)]
#[require(Ennemi, Vie = Vie(3))]
pub struct Slime;

commands.spawn(Slime); // a aussi `Ennemi` et `Vie(3)`
commands.spawn((Slime, Vie(10))); // `Vie(10)` : les composants donnés sont prioritaires
```

`Ennemi` seul utilise la valeur par défaut du composant, qui doit donc implémenter `Default`.
Bevy lui-même en fait un usage intensif : une caméra 2D (`Camera2d`) requiert une `Camera`, un
`Transform`, etc.

## Tester sans application

Pour tester les composants requis, pas besoin d'`App` ni de systèmes : un simple `World` suffit.

```rust
let mut world = World::new();
let slime = world.spawn(Slime).id();
assert_eq!(world.get::<Vie>(slime), Some(&Vie(3)));
```

Et pour vérifier qu'une entité a été détruite : `world.get_entity(entite).is_err()`.

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. Ajoutez l'attribut `#[require(...)]` à `Slime`.
2. `faire_apparaitre_la_vague` : crée `TailleVague` slimes.
3. `appliquer_le_poison` : retire un point de vie par image, et retire le poison quand il est
   épuisé.
4. `enrager_les_blesses` : ajoute `Enrage` aux ennemis qui n'ont plus qu'un point de vie.
5. `retirer_les_morts` : détruit les entités sans vie.
6. `plugin` : tout assembler, dans le bon ordre.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_03
```

## Pour aller plus loin

- Dans `enrager_les_blesses`, que se passerait-il sans le filtre `Without<Enrage>` ? Le jeu
  fonctionnerait toujours, mais une commande serait envoyée à chaque image pour chaque blessé.
- Les entités peuvent avoir des enfants : `commands.spawn((Parent, children![Enfant1, Enfant2]))`.
  Détruire le parent détruit aussi ses enfants. Voir `examples/ecs/hierarchy.rs`.
