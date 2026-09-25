# Apprendre Bevy, avec des tests prêts à l'emploi

Des tutoriels progressifs, en français, pour apprendre à créer des jeux avec
[Bevy](https://bevy.org). Chaque chapitre présente une notion, puis vous la faites pratiquer dans
un exercice... vérifié par des tests déjà écrits. Au dernier chapitre, vous assemblez tout dans
un vrai jeu.

## Prérequis

- Connaître les bases de Rust : variables, fonctions, structures, énumérations, boucles, `Option`
  (les chapitres 1 à 8 du [livre de Rust](https://jimskapt.github.io/rust-book-fr/) suffisent).
- Avoir installé Rust avec [rustup](https://rustup.rs), et cloné ce dépôt.

Sous Linux, Bevy a besoin de quelques paquets pour le dernier chapitre (la fenêtre et le son) :
voir [`docs/linux_dependencies.md`](../docs/linux_dependencies.md).

## Comment ça marche

Chaque chapitre est un dossier de [`src/`](src) qui contient :

| Fichier        | Contenu                                                     |
|----------------|-------------------------------------------------------------|
| `README.md`    | la leçon : à lire en premier                                |
| `exercice.rs`  | le code à compléter, là où se trouvent les `todo!()`        |
| `tests.rs`     | les tests, qui vérifient votre exercice : à lire aussi !    |
| `solution.rs`  | une solution, pour comparer ou en cas de blocage            |

Pour chaque chapitre :

1. Lisez la leçon (`README.md`).
2. Lancez les tests de l'exercice : ils échouent tous avec le message `not yet implemented`.

   ```sh
   cargo test -p bevy_tutorials --test exercices chapitre_01
   ```

3. Remplacez les `todo!()` de `exercice.rs` un par un, en relançant les tests à chaque fois.
   Chaque test qui passe vous dit qu'une étape est réussie.
4. Quand tous les tests passent, comparez avec `solution.rs` : il y a souvent plusieurs bonnes
   façons de faire !

Les tests sont les mêmes pour l'exercice et pour la solution. La solution passe tous les tests :

```sh
cargo test -p bevy_tutorials
```

## Sommaire

| Chapitre | Thème                                                   | Notions                                                            |
|----------|---------------------------------------------------------|--------------------------------------------------------------------|
| [1](src/chapitre_01/README.md) | Premiers pas                      | `App`, systèmes, `Startup`/`Update`, ressources, plugins           |
| [2](src/chapitre_02/README.md) | Entités, composants et requêtes   | ECS, `Component`, `Query`, `With`/`Without`, `Single`              |
| [3](src/chapitre_03/README.md) | Les commandes                     | `Commands`, créer/détruire, insérer/retirer, composants requis     |
| [4](src/chapitre_04/README.md) | Le temps                          | `Time`, `Transform`, `Timer`, contrôler le temps dans les tests    |
| [5](src/chapitre_05/README.md) | Le clavier                        | `ButtonInput<KeyCode>`, `pressed`/`just_pressed`, simuler le clavier |
| [6](src/chapitre_06/README.md) | Messages et observateurs          | `Message`, `MessageReader`/`Writer`, `Event`, `On<...>`            |
| [7](src/chapitre_07/README.md) | Les états du jeu                  | `States`, `OnEnter`, `run_if(in_state(...))`, `DespawnOnExit`      |
| [8](src/chapitre_08/README.md) | Organiser le jeu                  | `Plugin`, `SystemSet`, ordre des systèmes, tests d'intégration     |
| [9](src/chapitre_09/README.md) | Projet final : « Chasseur de pièces » | tout, dans un vrai jeu jouable                                 |

Le guide [**Tester un jeu Bevy**](TESTER.md) rassemble toutes les techniques de test des
tutoriels : gardez-le sous la main pour vos propres jeux.

## Jouer au projet final

```sh
# La solution :
cargo run -p bevy_tutorials --example chasseur_de_pieces --features jeu
# Votre version, une fois le chapitre 9 terminé :
cargo run -p bevy_tutorials --example chasseur_de_pieces --features jeu,exercice
```

La première compilation avec `--features jeu` prend plusieurs minutes : elle compile tout le
moteur de rendu. Les tests, eux, n'en ont pas besoin et compilent beaucoup plus vite.

## Conseils

- **Lisez les messages d'erreur jusqu'au bout.** Le compilateur Rust et Bevy expliquent souvent
  précisément le problème, et proposent parfois la solution.
- **Lisez les tests.** Ils décrivent exactement ce qui est attendu, et montrent comment tester vos
  propres jeux.
- **Un test à la fois.** Ajoutez le nom d'un test à la commande pour ne lancer que lui :
  `cargo test -p bevy_tutorials --test exercices le_poison_est_retire`.
- **Pas de panique** si un exercice vous bloque : lisez la solution, fermez-la, et réécrivez le
  code de mémoire.

## Et ensuite ?

- Les [exemples de Bevy](../examples/README.md) : plus de 300 petits programmes, un par
  fonctionnalité.
- La [documentation de Bevy](https://docs.rs/bevy) et le site [bevy.org/learn](https://bevy.org/learn).
- La [communauté Bevy](https://discord.gg/bevy), pour poser vos questions.
