//! Exercice du chapitre 9 : le projet final, « Chasseur de pièces ».
//!
//! Le joueur a 30 secondes pour ramasser un maximum de pièces, qui apparaissent au hasard dans
//! l'arène. Tout ce que vous avez appris dans les chapitres précédents sert ici !
//!
//! Les types, les constantes, le hasard et la fonction `direction` sont fournis. Remplacez chaque
//! `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_09
//! ```
//!
//! Et quand ils passent, jouez à votre version :
//!
//! ```text
//! cargo run -p bevy_tutorials --example chasseur_de_pieces --features jeu,exercice
//! ```

use bevy::{prelude::*, state::app::StatesPlugin};

/// La moitié de la largeur de l'arène, en pixels. L'arène est centrée sur l'origine.
pub const DEMI_LARGEUR_ARENE: f32 = 400.0;

/// La moitié de la hauteur de l'arène, en pixels.
pub const DEMI_HAUTEUR_ARENE: f32 = 300.0;

/// La taille du joueur (un carré), en pixels.
pub const TAILLE_JOUEUR: f32 = 40.0;

/// La taille des pièces, en pixels.
pub const TAILLE_PIECE: f32 = 20.0;

/// La vitesse du joueur, en pixels par seconde.
pub const VITESSE_JOUEUR: f32 = 300.0;

/// La distance en dessous de laquelle le joueur ramasse une pièce, en pixels.
pub const RAYON_DE_RAMASSAGE: f32 = 30.0;

/// La durée d'une partie, en secondes.
pub const DUREE_PARTIE: f32 = 30.0;

/// Le temps entre deux apparitions de pièces, en secondes.
pub const INTERVALLE_PIECES: f32 = 1.0;

/// Le nombre maximal de pièces dans l'arène.
pub const PIECES_MAX: usize = 5;

/// Les états du jeu.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EtatJeu {
    /// L'écran titre.
    #[default]
    Menu,
    /// Une partie est en cours.
    EnJeu,
    /// La partie est terminée : le score est affiché.
    FinDePartie,
}

/// Marque le joueur.
#[derive(Component)]
pub struct Joueur;

/// Marque les pièces.
#[derive(Component)]
pub struct Piece;

/// Le score de la partie en cours (ou de la dernière partie).
#[derive(Resource, Default)]
pub struct Score(pub u32);

/// Le meilleur score depuis le lancement du jeu.
#[derive(Resource, Default)]
pub struct MeilleurScore(pub u32);

/// Le temps de la partie : elle se termine quand ce minuteur est terminé.
#[derive(Resource)]
pub struct ChronoPartie(pub Timer);

impl Default for ChronoPartie {
    fn default() -> Self {
        Self(Timer::from_seconds(DUREE_PARTIE, TimerMode::Once))
    }
}

/// Le minuteur qui fait apparaître les pièces.
#[derive(Resource)]
pub struct MinuteurPieces(pub Timer);

impl Default for MinuteurPieces {
    fn default() -> Self {
        Self(Timer::from_seconds(INTERVALLE_PIECES, TimerMode::Repeating))
    }
}

/// Un message envoyé quand le joueur ramasse une pièce.
#[derive(Message)]
pub struct PieceRamassee;

/// Un générateur de nombres pseudo-aléatoires (*xorshift*), simple et reproductible : avec la
/// même graine, il donne toujours la même suite de nombres, ce qui rend les tests fiables.
///
/// La graine (le nombre de départ) ne doit pas être nulle.
#[derive(Resource)]
pub struct Hasard(pub u32);

impl Default for Hasard {
    fn default() -> Self {
        Self(0x2545_F491)
    }
}

impl Hasard {
    /// Renvoie un nombre entre 0 (inclus) et 1 (exclus).
    pub fn nombre(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        // Les 24 bits de poids fort donnent un `f32` exact entre 0 et 1.
        (x >> 8) as f32 / (1 << 24) as f32
    }

    /// Renvoie une position au hasard dans l'arène, à au moins [`TAILLE_PIECE`] pixels des bords.
    pub fn position_dans_l_arene(&mut self) -> Vec2 {
        Vec2::new(
            (self.nombre() * 2.0 - 1.0) * (DEMI_LARGEUR_ARENE - TAILLE_PIECE),
            (self.nombre() * 2.0 - 1.0) * (DEMI_HAUTEUR_ARENE - TAILLE_PIECE),
        )
    }
}

/// Renvoie la direction choisie avec les flèches ou les touches WASD (ZQSD en AZERTY), de
/// longueur 1 ou nulle.
pub fn direction(clavier: &ButtonInput<KeyCode>) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if clavier.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction.y += 1.0;
    }
    if clavier.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction.y -= 1.0;
    }
    if clavier.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction.x -= 1.0;
    }
    if clavier.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction.x += 1.0;
    }
    direction.normalize_or_zero()
}

/// Dans le menu ou à la fin d'une partie : lance une partie quand la touche Entrée vient d'être
/// enfoncée.
pub fn lancer_la_partie() {
    todo!()
}

/// Au début d'une partie : fait apparaître le joueur au centre (il disparaîtra à la fin de la
/// partie), et remet à zéro le score et les deux minuteurs.
///
/// Indice : `*chrono = ChronoPartie::default();` remplace toute la ressource.
pub fn preparer_la_partie(mut commands: Commands) {
    todo!()
}

/// Déplace le joueur selon le clavier, à [`VITESSE_JOUEUR`] pixels par seconde, sans qu'il puisse
/// sortir de l'arène : son centre reste à au moins `TAILLE_JOUEUR / 2` pixels des bords.
///
/// Indice : `Vec2::clamp(min, max)` limite chaque coordonnée d'un vecteur.
pub fn deplacer_le_joueur() {
    todo!()
}

/// Toutes les [`INTERVALLE_PIECES`] secondes, fait apparaître une pièce à une position au
/// [`Hasard`] dans l'arène, sauf s'il y a déjà [`PIECES_MAX`] pièces. Les pièces disparaissent à
/// la fin de la partie.
pub fn faire_apparaitre_des_pieces(mut commands: Commands) {
    todo!()
}

/// Détruit les pièces proches du joueur (à moins de [`RAYON_DE_RAMASSAGE`] pixels), et envoie un
/// message [`PieceRamassee`] pour chacune.
pub fn ramasser_les_pieces(mut commands: Commands) {
    todo!()
}

/// Ajoute un point au [`Score`] pour chaque pièce ramassée.
pub fn compter_les_points(mut score: ResMut<Score>) {
    todo!()
}

/// Fait avancer le [`ChronoPartie`], et termine la partie quand il est écoulé.
pub fn decompter_le_temps(temps: Res<Time>) {
    todo!()
}

/// À la fin d'une partie : met à jour le [`MeilleurScore`].
pub fn enregistrer_le_meilleur_score() {
    todo!()
}

/// Le plugin du jeu : toute la logique, sans l'affichage.
pub struct JeuPlugin;

impl Plugin for JeuPlugin {
    /// Doit ajouter :
    /// - le `StatesPlugin`, s'il n'est pas déjà là (`app.is_plugin_added::<StatesPlugin>()`) ;
    /// - l'état [`EtatJeu`], la ressource `ButtonInput<KeyCode>` (pour les tests), le message et
    ///   les ressources ;
    /// - [`preparer_la_partie`] en entrant dans [`EtatJeu::EnJeu`], et
    ///   [`enregistrer_le_meilleur_score`] en entrant dans [`EtatJeu::FinDePartie`] ;
    /// - [`lancer_la_partie`] quand on n'est pas en jeu (indice : `not(in_state(...))`) ;
    /// - les cinq systèmes de la partie, dans l'ordre de ce fichier, seulement en jeu.
    fn build(&self, app: &mut App) {
        todo!()
    }
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
