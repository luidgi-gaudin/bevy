# Chapitre 14 — Le maniement : visée, recul et dispersion

Objectifs :

- épauler son arme pour viser (*ADS*), comme dans Call of Duty ;
- donner à chaque arme un **recul** que les joueurs apprennent à maîtriser ;
- disperser les balles selon la situation, avec un hasard **reproductible**.

C'est ce qui donne sa « sensation » à un FPS : deux jeux avec les mêmes armes, mais un maniement
différent, ne se jouent pas du tout pareil.

## Épauler

En maintenant le bouton droit, le joueur épaule son arme : en 0,2 seconde, la visée
(`Visee::progression`) passe de 0 (à la hanche) à 1 (épaulé). Tout ce qui dépend de la visée
varie en douceur avec elle :

| Effet                  | À la hanche | Épaulé  |
|------------------------|-------------|---------|
| Champ de vision        | 70°         | 45° (l'image est agrandie) |
| Vitesse de déplacement | × 1         | × 0,6, et pas de sprint |
| Dispersion             | 3°          | 0,3°    |
| Recul                  | × 1         | × 0,5   |

C'est le choix permanent d'un joueur de Call of Duty : tirer vite à la hanche en bougeant, ou
épauler pour être précis, mais plus lent et plus vulnérable.

> Le champ de vision s'applique à la caméra (`Projection`), qui n'existe que dans le jeu avec
> rendu : la fonction `champ_de_vision` le calcule, et l'arène jouable l'applique.

## Le recul

À chaque balle, l'arme secoue le regard du tireur selon un **motif** fixe : d'abord vers le haut,
puis de gauche à droite. Comme le motif est toujours le même, les bons joueurs apprennent à le
compenser en tirant la souris dans l'autre sens. C'est une compétence, pas du hasard : essentiel
dans un jeu compétitif.

```rust
for tir in tirs.read() {
    let secousse = recul.motif[recul.balle] * reduction_en_visant;
    orientation.tangage += secousse.y.to_radians();
    orientation.lacet -= secousse.x.to_radians();
    recul.balle += 1;
}
```

Après une courte pause sans tirer, le motif recommence au début : c'est ce qui rend les rafales
courtes plus précises que les longues.

## La dispersion, et le hasard reproductible

Même en visant parfaitement, les balles partent un peu au hasard dans un **cône** autour du
regard : très étroit en visant immobile, large à la hanche, en courant ou en sautant. Cela
récompense les joueurs qui s'arrêtent pour tirer.

Mais pour un jeu en réseau, le hasard pose un problème : le serveur et le joueur doivent calculer
**exactement** la même déviation, sinon le joueur verrait sa balle toucher, et le serveur la
compterait manquée. La solution : un hasard **déterminé par une graine**, calculée à partir du
tireur et du numéro de la balle.

```rust
let direction = devier(
    direction_du_regard(orientation),
    dispersion(visee.progression, physique),
    graine_de_balle(tireur, etat.balles_tirees),
);
```

`hasard(graine)` est une fonction de hachage : de petits changements de la graine changent
complètement le résultat, mais la même graine donne toujours le même nombre. Le test
`deux_parties_identiques_donnent_exactement_les_memes_tirs` le prouve.

## La durée de vie des messages

Un piège à connaître : les messages ne sont gardés que **deux images**. Un test qui voudrait
compter toutes les balles d'une longue rafale en lisant `Messages<Tir>` à la fin n'en trouverait
que les dernières. Il faut les lire au fur et à mesure, avec un système :

```rust
fn enregistrer_les_tirs(mut tirs: MessageReader<Tir>, mut enregistres: ResMut<TirsEnregistres>) {
    enregistres.0.extend(tirs.read().map(|tir| tir.direction));
}
```

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `champ_de_vision`, `epauler` et `dispersion`.
2. `devier` : la déviation dans le cône (suivez les indices).
3. `lire_la_visee`, `ralentir_en_visant` et `epauler_les_armes`.
4. `tirer_avec_dispersion` : le `tirer` du chapitre 13, avec la dispersion.
5. `appliquer_le_recul` et `plugin`.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_14
```

## Pour aller plus loin

- Dans de nombreux jeux, la sensibilité de la souris diminue en visant, dans le même rapport que
  le champ de vision, pour garder la même précision de visée.
- Un fusil de précision n'a presque pas de dispersion en visant, mais une énorme à la hanche : les
  joueurs doivent épauler. Créez son `Recul` et testez son maniement.
