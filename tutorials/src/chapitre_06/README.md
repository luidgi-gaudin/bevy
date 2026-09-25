# Chapitre 6 — Les messages et les observateurs

Objectifs :

- faire communiquer des systèmes sans qu'ils se connaissent, avec des **messages** ;
- réagir immédiatement à un **événement** avec des **observateurs** ;
- savoir choisir entre les deux.

## Les messages

Quand le joueur ramasse une pièce, plusieurs choses doivent se passer : le score augmente, un son
est joué, le journal est mis à jour... Plutôt que d'écrire tout cela dans le système qui détecte
le ramassage, il envoie un **message**, et chaque système intéressé le lit de son côté.

```rust
#[derive(Message)]
pub struct PieceRamassee {
    pub valeur: u32,
}

// L'écrivain :
fn ramasser_les_pieces(mut pieces_ramassees: MessageWriter<PieceRamassee>, ...) {
    pieces_ramassees.write(PieceRamassee { valeur: 5 });
}

// Un lecteur :
fn compter_les_points(mut pieces_ramassees: MessageReader<PieceRamassee>, mut score: ResMut<Score>) {
    for piece in pieces_ramassees.read() {
        score.0 += piece.valeur;
    }
}
```

Le type de message doit être déclaré à l'application : `app.add_message::<PieceRamassee>()`.

- Chaque lecteur reçoit **tous** les messages, et chacun ne les lit **qu'une fois**.
- Les messages sont gardés deux images, puis effacés : un système qui ne les lit pas à temps les
  perd.
- Pour lire les messages dans l'image où ils sont envoyés, le lecteur doit s'exécuter après
  l'écrivain (avec `.chain()`, par exemple) ; sinon, il les lira à l'image suivante.

## Les observateurs

Un **observateur** est un système qui s'exécute **immédiatement** quand un **événement** est
déclenché, au lieu d'attendre son tour dans une planification.

```rust
#[derive(Event)]
pub struct EnnemiVaincu;

fn recompenser(_vaincu: On<EnnemiVaincu>, mut score: ResMut<Score>) {
    score.0 += 10;
}

app.add_observer(recompenser);
commands.trigger(EnnemiVaincu); // `recompenser` s'exécute
```

Le premier paramètre, `On<E>`, donne accès à l'événement ; les suivants sont ceux d'un système
ordinaire.

### Les événements d'entité

Un `EntityEvent` vise une entité précise, désignée par son champ `entity` :

```rust
#[derive(EntityEvent)]
pub struct Touche {
    pub entity: Entity,
    pub degats: u32,
}

fn subir_les_coups(touche: On<Touche>, mut vies: Query<&mut Vie>) {
    if let Ok(mut vie) = vies.get_mut(touche.entity) {
        vie.0 = vie.0.saturating_sub(touche.degats);
    }
}
```

On peut aussi attacher un observateur à une seule entité :
`commands.spawn(Boss).observe(quand_le_boss_est_touche)`.

### Les événements du cycle de vie

Bevy déclenche lui-même des événements quand un composant est ajouté (`Add`), inséré (`Insert`)
ou retiré (`Remove`) d'une entité :

```rust
fn compter_les_apparitions(ajout: On<Add<Piece>>, mut apparues: ResMut<PiecesApparues>) {
    apparues.0 += 1; // `ajout.entity` est la nouvelle pièce
}
```

C'est idéal pour préparer une entité dès son apparition : au chapitre 9, c'est ainsi que le jeu
ajoute une image aux pièces, sans que la logique du jeu ne s'occupe de l'affichage.

## Messages ou observateurs ?

| Messages                                           | Observateurs                                     |
|----------------------------------------------------|--------------------------------------------------|
| traités à un moment précis de l'image, en groupe   | traités immédiatement, un par un                 |
| efficaces pour beaucoup d'événements               | idéaux pour des réactions rares ou en chaîne     |
| plusieurs lecteurs, qui lisent à leur rythme       | peuvent viser une seule entité                   |

## Dans les tests

```rust
// Envoyer un message soi-même :
app.world_mut().write_message(PieceRamassee { valeur: 3 });

// Lire les messages envoyés :
let messages = app.world().resource::<Messages<PieceRamassee>>();
let envoyes: Vec<_> = messages.get_cursor().read(messages).collect();

// Déclencher un événement : les observateurs s'exécutent tout de suite.
app.world_mut().trigger(Touche { entity: ennemi, degats: 1 });
// Les commandes envoyées par les observateurs sont appliquées au prochain `flush`.
app.world_mut().flush();
```

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `ramasser_les_pieces` : détruit les pièces proches, et envoie un message pour chacune.
2. `compter_les_points` et `tenir_le_journal` : deux lecteurs du même message.
3. `subir_les_coups` et `recompenser` : deux observateurs qui s'enchaînent.
4. `compter_les_apparitions` : un observateur du cycle de vie.
5. `plugin` : tout assembler.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_06
```

## Pour aller plus loin

- Dans le plugin, retirez `.chain()` et lancez les tests : que se passe-t-il ? Rien ne dit plus à
  Bevy d'exécuter les lecteurs après l'écrivain : le score peut n'augmenter qu'à l'image suivante.
- Les événements d'entité peuvent se **propager** aux parents d'une entité : voir
  `examples/ecs/observer_propagation.rs`.
