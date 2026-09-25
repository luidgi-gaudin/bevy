# Chapitre 11 — Se déplacer à tick fixe

Objectifs :

- simuler le jeu à un rythme fixe, le **tick rate**, comme les serveurs des FPS compétitifs ;
- programmer des déplacements nerveux : accélération, friction, sprint, saut ;
- ne jamais perdre une touche, et garder un affichage fluide grâce à l'**interpolation** ;
- vérifier par un test que la simulation est **déterministe**.

## Pourquoi un tick fixe ?

Au chapitre 4, on multipliait les vitesses par la durée de l'image. C'est suffisant pour un jeu
simple, mais pas pour un FPS compétitif :

- avec des images de durées différentes, les arrondis diffèrent : deux joueurs qui font
  exactement les mêmes gestes n'arrivent pas exactement au même endroit ;
- un saut ne monte pas tout à fait à la même hauteur à 60 et à 300 images par seconde (c'était un
  vrai problème de Quake 3, où les joueurs à 125 images par seconde sautaient plus haut !) ;
- en réseau, le serveur et les joueurs doivent simuler **exactement** la même chose.

La solution : simuler le jeu par **ticks** de durée fixe. Les serveurs de Valorant tournent à 128
ticks par seconde, ceux de nombreux jeux à 64. Dans Bevy, c'est la planification `FixedUpdate` :

```rust
app.insert_resource(Time::<Fixed>::from_hz(128.0))
    .add_systems(FixedUpdate, simuler_le_mouvement);
```

À chaque image, Bevy exécute autant de ticks que nécessaire pour rattraper le temps écoulé :
aucun à 300 images par seconde, parfois deux à 60. Dans `FixedUpdate`, `Res<Time>` donne toujours
la durée d'un tick (1/128 s).

Le test `le_mouvement_ne_depend_pas_des_images_par_seconde` le prouve : à 40 ou à 100 images par
seconde, le joueur arrive **exactement** au même endroit, au bit près.

## Des déplacements nerveux

Les FPS compétitifs reprennent souvent le modèle de Quake, dont descendent Counter-Strike et
Call of Duty :

- **l'accélération** rapproche la vitesse de la vitesse voulue, sans la dépasser ;
- **la friction** freine au sol, et arrête net le joueur qui lâche les touches ;
- **en l'air**, l'accélération est faible : on garde son élan pendant un saut ;
- **la gravité** est plus forte que sur Terre, pour des sauts courts et nerveux.

```rust
pub fn accelerer(vitesse: Vec3, direction: Vec3, vitesse_voulue: f32, acceleration: f32, dt: f32) -> Vec3 {
    let manque = vitesse_voulue - vitesse.dot(direction);
    if manque <= 0.0 {
        return vitesse;
    }
    vitesse + direction * (acceleration * vitesse_voulue * dt).min(manque)
}
```

Tout le calcul d'un tick est dans une fonction ordinaire, `tick_de_mouvement`, qui prend l'état
du joueur et renvoie le nouveau. Elle se teste sans application, tick par tick, et le même code
pourra tourner sur le serveur.

> Pourquoi `ops::sin_cos` plutôt que `f32::sin_cos` ? Les fonctions de `f32` utilisent celles du
> système, qui peuvent différer très légèrement d'un ordinateur à l'autre. Celles de
> `bevy::math::ops` donnent exactement le même résultat partout : indispensable pour que tous les
> joueurs simulent la même partie.

## Lire les entrées au bon moment

Une image se déroule ainsi : `PreUpdate` (Bevy lit le clavier et la souris), puis
`RunFixedMainLoop` (les ticks de `FixedUpdate`), puis `Update`, puis le rendu. Si on lisait le
clavier dans `Update`, les ticks utiliseraient les touches de l'image **précédente** : une image
de retard, inacceptable pour un jeu compétitif. On les lit donc juste avant les ticks :

```rust
app.add_systems(
    RunFixedMainLoop,
    lire_le_clavier.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
);
```

Il reste un piège : à 300 images par seconde, la plupart des images n'exécutent **aucun** tick.
Si le joueur appuie brièvement sur Espace pendant une de ces images, `just_pressed` est vrai pour
cette image seulement, et le saut serait perdu. La commande de saut reste donc **en attente**
jusqu'au prochain tick, qui la consomme :

```rust
if clavier.just_pressed(KeyCode::Space) {
    commandes.saut = true; // remis à `false` par le tick qui l'utilise
}
```

## Un affichage fluide : l'interpolation

Les ticks ont lieu 128 fois par seconde, mais l'écran peut en afficher 240 : si on affichait la
position du dernier tick, le joueur avancerait par saccades. On affiche donc une position
**interpolée** entre les deux derniers ticks, selon le temps écoulé depuis le dernier :

```rust
let avancement = temps_fixe.overstep_fraction(); // entre 0 et 1
transform.translation = physique.position_precedente.lerp(physique.position, avancement);
```

L'état simulé (`Physique`) est ainsi séparé de ce qui est affiché (`Transform`).

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `axes_au_sol`, `frotter` et `accelerer` : les briques du mouvement.
2. `tick_de_mouvement` : un tick complet (suivez les étapes de sa documentation).
3. `lire_le_clavier` : sans perdre les sauts.
4. `simuler_le_mouvement` et `interpoler`.
5. `plugin` : chaque système au bon endroit de l'image.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_11
```

## Pour aller plus loin

- Les joueurs de Counter-Strike « strafent » en l'air pour gagner de la vitesse (*air strafing*,
  *bunny hopping*) : c'est une conséquence de `accelerer`, qui ne limite la vitesse que dans la
  direction voulue. Essayez d'écrire un test qui le montre !
- Call of Duty ajoute des glissades (*slide*) : une impulsion vers l'avant, avec une friction
  réduite pendant un court instant.
