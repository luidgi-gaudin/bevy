//! Exercice du chapitre 4 : le temps, les minuteurs et les déplacements réguliers.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_04
//! ```

use bevy::prelude::*;

/// La vitesse d'une entité, en pixels par seconde.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Vitesse(pub Vec2);

/// Marque les pièces à ramasser.
#[derive(Component)]
pub struct Piece;

/// Le temps qu'il reste à vivre à une entité : elle disparaît quand le minuteur est terminé.
#[derive(Component)]
pub struct DureeDeVie(pub Timer);

/// Le minuteur qui fait apparaître une pièce toutes les 2 secondes.
#[derive(Resource)]
pub struct MinuteurPieces(pub Timer);

impl Default for MinuteurPieces {
    /// Un minuteur de 2 secondes, qui recommence à chaque fois qu'il est terminé.
    fn default() -> Self {
        todo!()
    }
}

/// Le temps écoulé depuis le lancement du jeu, en secondes.
#[derive(Resource, Default)]
pub struct Chrono(pub f32);

/// La durée de vie des pièces, en secondes.
pub const DUREE_DE_VIE_PIECES: f32 = 5.0;

/// Déplace chaque entité selon sa [`Vitesse`], en tenant compte de la durée de l'image.
///
/// Indices :
/// - `Res<Time>` donne le temps ; `temps.delta_secs()` est la durée de l'image, en secondes ;
/// - la position d'une entité est dans son `Transform` : `transform.translation` est un `Vec3`,
///   et `vec2.extend(0.0)` transforme un `Vec2` en `Vec3`.
pub fn deplacer() {
    todo!()
}

/// Fait apparaître une pièce toutes les 2 secondes, au centre, avec une durée de vie de
/// [`DUREE_DE_VIE_PIECES`] secondes.
///
/// Indices : `minuteur.0.tick(temps.delta())` fait avancer le minuteur, et
/// `minuteur.0.just_finished()` dit s'il vient de se terminer.
pub fn faire_apparaitre_des_pieces(mut commands: Commands, mut minuteur: ResMut<MinuteurPieces>) {
    todo!()
}

/// Fait avancer la [`DureeDeVie`] des entités, et détruit celles dont la durée de vie est
/// terminée.
pub fn faire_disparaitre(mut commands: Commands) {
    todo!()
}

/// Écrit le temps écoulé depuis le lancement du jeu dans le [`Chrono`].
pub fn mettre_a_jour_le_chrono(mut chrono: ResMut<Chrono>) {
    todo!()
}

/// Le plugin du chapitre : les deux ressources, et les quatre systèmes à chaque image.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
