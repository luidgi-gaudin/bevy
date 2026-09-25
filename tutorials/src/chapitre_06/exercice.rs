//! Exercice du chapitre 6 : communiquer entre systèmes avec les messages et les observateurs.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_06
//! ```

use bevy::prelude::*;

/// Marque le joueur.
#[derive(Component)]
pub struct Joueur;

/// Une pièce à ramasser, qui rapporte `valeur` points.
#[derive(Component)]
pub struct Piece {
    /// Le nombre de points que rapporte la pièce.
    pub valeur: u32,
}

/// Les points de vie d'une entité.
#[derive(Component, Debug, PartialEq)]
pub struct Vie(pub u32);

/// Le score du joueur.
#[derive(Resource, Default)]
pub struct Score(pub u32);

/// Les événements marquants de la partie, du plus ancien au plus récent.
#[derive(Resource, Default)]
pub struct Journal(pub Vec<String>);

/// Le nombre de pièces apparues depuis le début de la partie.
#[derive(Resource, Default)]
pub struct PiecesApparues(pub u32);

/// La distance en dessous de laquelle le joueur ramasse une pièce, en pixels.
pub const RAYON_DE_RAMASSAGE: f32 = 32.0;

/// Les points gagnés pour chaque ennemi vaincu.
pub const POINTS_PAR_ENNEMI: u32 = 10;

/// Un message envoyé quand le joueur ramasse une pièce.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct PieceRamassee {
    /// La valeur de la pièce ramassée.
    pub valeur: u32,
}

/// Un coup porté à une entité : un événement qui vise cette entité.
#[derive(EntityEvent)]
pub struct Touche {
    /// L'entité touchée.
    pub entity: Entity,
    /// Le nombre de points de vie perdus.
    pub degats: u32,
}

/// Un événement déclenché quand un ennemi est vaincu.
#[derive(Event)]
pub struct EnnemiVaincu;

/// Détruit les pièces proches du joueur (à moins de [`RAYON_DE_RAMASSAGE`] pixels), et envoie un
/// message [`PieceRamassee`] pour chacune.
///
/// Indices : `MessageWriter<PieceRamassee>` envoie des messages avec `write`, et
/// `a.distance(b)` donne la distance entre deux `Vec3`.
pub fn ramasser_les_pieces(mut commands: Commands) {
    todo!()
}

/// Ajoute la valeur des pièces ramassées au [`Score`].
///
/// Indice : `MessageReader<PieceRamassee>` lit les messages avec `read`.
pub fn compter_les_points(mut score: ResMut<Score>) {
    todo!()
}

/// Écrit `"+<valeur>"` dans le [`Journal`] pour chaque pièce ramassée.
pub fn tenir_le_journal(mut journal: ResMut<Journal>) {
    todo!()
}

/// Un observateur de [`Touche`] : retire les dégâts à la [`Vie`] de l'entité touchée. Si elle n'a
/// plus de points de vie, la détruit et déclenche [`EnnemiVaincu`].
///
/// Si l'entité touchée n'a pas de [`Vie`], il ne se passe rien.
///
/// Indices : `touche.entity` est l'entité touchée, `vies.get_mut(entite)` donne sa vie (ou une
/// erreur), et `commands.trigger(EnnemiVaincu)` déclenche un événement.
pub fn subir_les_coups(touche: On<Touche>, mut commands: Commands) {
    todo!()
}

/// Un observateur de [`EnnemiVaincu`] : ajoute [`POINTS_PAR_ENNEMI`] au [`Score`].
pub fn recompenser(_vaincu: On<EnnemiVaincu>, mut score: ResMut<Score>) {
    todo!()
}

/// Un observateur de l'ajout du composant [`Piece`] à une entité : compte les pièces apparues.
pub fn compter_les_apparitions(_ajout: On<Add<Piece>>, mut apparues: ResMut<PiecesApparues>) {
    todo!()
}

/// Le plugin du chapitre : le message, les trois ressources, les trois systèmes (les lecteurs de
/// messages après l'écrivain) et les trois observateurs.
///
/// Indices : `app.add_message::<M>()` et `app.add_observer(observateur)`.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
