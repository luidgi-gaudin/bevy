# Chapitre 17 — La compensation de latence

Objectifs :

- comprendre comment les FPS en ligne cachent la latence du réseau ;
- enregistrer l'historique des positions à chaque tick ;
- revenir dans le temps pour juger un tir tel que le tireur l'a vu ;
- limiter ce retour, pour rester juste envers la cible.

## Le problème : chacun voit le passé

Dans un FPS en ligne, un **serveur** fait autorité : c'est lui qui simule la partie et décide si
une balle touche. Chaque joueur lui envoie ses commandes et reçoit en retour la position des
autres. Mais un message met du temps à traverser Internet : avec 50 millisecondes de *ping*, ce
qu'un joueur voit à l'écran a au moins 50 millisecondes de retard, souvent plus.

Imaginez un adversaire qui court à 7,5 m/s. En 100 millisecondes, il parcourt 75 centimètres :
plus que la largeur de son torse (70 cm), et trois fois celle de sa tête. Le joueur vise
parfaitement l'adversaire qu'il voit et tire.
Sur le serveur, l'adversaire est déjà ailleurs, et la balle le rate. Sans correction, il
faudrait viser **devant** les cibles, en devinant la latence : injouable.

## Les trois techniques des FPS en ligne

1. **La prédiction** : le joueur n'attend pas le serveur pour se déplacer. Son jeu simule
   aussitôt ses propres commandes, avec exactement le même code que le serveur. C'est pour cela
   que le chapitre 11 a rendu le mouvement **déterministe** : si la prédiction et le serveur
   calculent la même chose, le serveur n'a jamais à corriger le joueur.
2. **L'interpolation** : les autres joueurs sont affichés entre les deux dernières positions
   reçues du serveur, pour un mouvement fluide malgré les paquets espacés. Comme la fonction
   `interpoler` du chapitre 11, mais entre des ticks reçus du réseau.
3. **La compensation de latence**, le sujet de ce chapitre : le serveur juge chaque tir **dans le
   passé**, tel que le tireur l'a vu.

## Revenir dans le temps

À chaque tick, le serveur enregistre la position de chaque personnage dans son `Historique` :

```rust
pub fn enregistrer(&mut self, tick: u32, position: Vec3) {
    self.positions.push_back((tick, position));
    while self.positions.len() > TICKS_D_HISTORIQUE {
        self.positions.pop_front();
    }
}
```

Une `VecDeque` est une file : on ajoute à la fin, on retire au début, sans rien déplacer en
mémoire. On garde une seconde d'historique, soit 128 positions.

Quand un tir arrive, le serveur connaît la `Latence` du tireur, en ticks. Il replace les hitbox
de chaque cible à la position qu'elle avait `latence` ticks plus tôt, tire le rayon du
chapitre 13 contre ces hitbox du passé, puis inflige les dégâts dans le présent. Le tireur a vu
la cible, visé la cible, touché la cible : c'est la règle de Counter-Strike, Valorant et Call of
Duty.

```rust
pub fn position_vue(historique: &Historique, actuelle: Vec3, tick: u32, latence: u32) -> Vec3
```

Remarquez que `tirer_avec_compensation` ne déplace pas vraiment les cibles : elle calcule
seulement où placer leurs hitbox pour ce rayon. Les autres systèmes ne voient rien, et deux
tireurs de latences différentes, pendant le même tick, sont chacun jugés selon ce qu'ils ont vu.

## Le prix à payer

La compensation de latence avantage le tireur, au détriment de la cible. Quelques
conséquences bien connues des joueurs :

- **mourir derrière un mur** : la cible s'est mise à couvert sur son écran, mais le tireur la
  voyait encore à découvert, et le serveur lui donne raison ;
- **l'avantage de celui qui surgit** (*peeker's advantage*) : un joueur qui sort de derrière un
  mur voit son adversaire avant que celui-ci ne le voie ;
- **les connexions lentes** : sans limite, un joueur avec une seconde de latence pourrait toucher
  des cibles où elles étaient une seconde plus tôt.

D'où la constante `RETOUR_MAX` : le serveur ne remonte jamais plus de 26 ticks, environ
200 millisecondes, en arrière. Au-delà, c'est au joueur de viser devant sa cible. Le test
`au_dela_de_200_ms_la_compensation_s_arrete` le vérifie.

## Tout est tick

Le `Tick` est compté dans `FixedFirst`, au tout début de chaque tick, avant le mouvement. Puis,
dans `FixedUpdate`, après les collisions :

```text
enregistrer_les_positions → epauler_les_armes → tirer_avec_compensation → appliquer_le_recul
```

Les positions sont enregistrées **avant** le tir : la position du tick en cours est déjà dans
l'historique quand un tireur sans latence tire. Dans un vrai jeu en réseau, le client
enverrait avec chaque tir le numéro du tick qu'il affichait, plutôt qu'une latence mesurée par
le serveur.

## À vous de jouer

Complétez [`exercice.rs`](exercice.rs) :

1. `Historique::enregistrer` et `Historique::position_au_tick` (cherchez depuis la fin de la
   file : `iter().rev()`).
2. `position_vue`.
3. `compter_les_ticks` et `enregistrer_les_positions`.
4. `tirer_avec_compensation` : repartez de `tirer_avec_dispersion` (chapitre 14).
5. `plugin`.

```sh
cargo test -p bevy_tutorials --test exercices chapitre_17
```

Les tests `le_tir_touche_la_cible_la_ou_le_tireur_la_voyait` et `sans_latence_la_meme_balle_rate`
tirent la même balle, sur une cible qui vient de s'écarter : seul le tireur avec 10 ticks de
latence la touche, car il la voyait encore devant lui.

## Pour aller plus loin

- Ce chapitre simule un serveur, sans réseau. Pour un vrai jeu en ligne, les bibliothèques
  [`lightyear`](https://github.com/cBournhonesque/lightyear) et
  [`bevy_replicon`](https://github.com/projectharmonia/bevy_replicon) gèrent les connexions, la
  réplication, la prédiction et l'interpolation. Leur documentation parle des mêmes notions que
  ce chapitre.
- Les hitbox ne sont pas que des positions : si les personnages s'accroupissent, enregistrez
  aussi leur posture dans l'historique.
- Mesurez la latence de chaque joueur (le temps d'un aller-retour, divisé par deux) et gardez
  une moyenne glissante : une latence qui varie (*jitter*) ne doit pas faire trembler les hitbox.
