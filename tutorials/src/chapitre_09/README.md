# Chapitre 9 — Projet final : « Chasseur de pièces »

Il est temps de tout assembler dans un vrai jeu ! Le joueur a **30 secondes** pour ramasser un
maximum de pièces, qui apparaissent au hasard dans l'arène. Le jeu a un menu, un score, un
meilleur score, et on peut rejouer.

```sh
# Jouer à la solution :
cargo run -p bevy_tutorials --example chasseur_de_pieces --features jeu
```

## Ce que vous allez utiliser

| Chapitre | Notion                     | Dans le jeu                                          |
|----------|----------------------------|------------------------------------------------------|
| 1        | ressources, plugins        | `Score`, `MeilleurScore`, `JeuPlugin`                |
| 2        | composants, requêtes       | `Joueur`, `Piece`, compter les pièces                |
| 3        | commandes                  | créer et détruire les pièces                         |
| 4        | le temps, les minuteurs    | `ChronoPartie`, `MinuteurPieces`, les déplacements   |
| 5        | le clavier                 | se déplacer, lancer une partie                       |
| 6        | les messages               | `PieceRamassee`, lu par le score                     |
| 7        | les états                  | `Menu`, `EnJeu`, `FinDePartie`, `DespawnOnExit`      |
| 8        | l'ordre des systèmes       | les systèmes de la partie, enchaînés                 |

## Séparer la logique de l'affichage

Le fichier [`exercice.rs`](exercice.rs) ne contient **aucun** affichage : ni caméra, ni image, ni
texte. Il ne décrit que les règles du jeu. C'est ce qui permet de le tester entièrement sans
fenêtre, en jouant des parties complètes en quelques millisecondes.

L'affichage est ajouté par l'exemple
[`examples/chasseur_de_pieces.rs`](../../examples/chasseur_de_pieces.rs), avec des observateurs
qui donnent une apparence aux entités dès qu'elles apparaissent :

```rust
fn dessiner_les_pieces(ajout: On<Add<Piece>>, mut commands: Commands) {
    commands.entity(ajout.entity).insert(Sprite::from_color(
        Color::srgb(1.0, 0.8, 0.2),
        Vec2::splat(TAILLE_PIECE),
    ));
}
```

Cette séparation est un excellent réflexe pour vos propres jeux : on peut changer tout
l'affichage (des images, des animations, de la 3D...) sans toucher aux règles, ni aux tests.

## Un hasard reproductible

Les pièces apparaissent « au hasard ». Mais un test qui dépend du hasard peut réussir une fois et
échouer la suivante ! La ressource `Hasard` est un générateur **pseudo-aléatoire** : il calcule
une suite de nombres qui semblent aléatoires, mais qui est toujours la même pour un même nombre
de départ (la graine). Les tests sont donc reproductibles. Dans un jeu publié, on choisirait une
graine différente à chaque lancement (à partir de l'heure, par exemple).

## Le déroulement d'une partie

```text
      Entrée              30 secondes écoulées
Menu ───────▶ EnJeu ──────────────────────────▶ FinDePartie
                ▲                                   │
                └──────────────── Entrée ───────────┘
```

- En entrant dans `EnJeu` : le joueur apparaît au centre, le score et les minuteurs repartent de
  zéro.
- Pendant `EnJeu`, dans cet ordre : déplacer le joueur, faire apparaître les pièces, ramasser les
  pièces, compter les points, décompter le temps.
- En entrant dans `FinDePartie` : le meilleur score est mis à jour. Le joueur et les pièces
  disparaissent grâce à `DespawnOnExit`.

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs). Commencez par le plugin (`JeuPlugin::build`), sans quoi
aucun test ne peut s'exécuter, puis écrivez les systèmes un par un, en relançant les tests :

```sh
cargo test -p bevy_tutorials --test exercices chapitre_09
```

Les tests de [`tests.rs`](tests.rs) jouent des parties entières : lisez-les, ils montrent comment
tester un jeu complet. Quand ils passent tous, jouez à votre version :

```sh
cargo run -p bevy_tutorials --example chasseur_de_pieces --features jeu,exercice
```

## Pour aller plus loin

Quelques idées pour améliorer le jeu, en écrivant d'abord un test pour chacune :

- des pièces rares qui valent 5 points (un champ `valeur` dans `Piece`, comme au chapitre 6) ;
- des ennemis qui poursuivent le joueur et lui font perdre des points ;
- une pause avec la touche Échap (voir les sous-états, au chapitre 7) ;
- des sons, avec `AudioPlayer` : ajoutez `"bevy/audio"` à la fonctionnalité `jeu` du
  `Cargo.toml` des tutoriels, et voir `examples/audio/` ;
- une vraie image pour le joueur, avec `Sprite::from_image(asset_server.load("joueur.png"))`
  (voir `examples/2d/sprite.rs`).
