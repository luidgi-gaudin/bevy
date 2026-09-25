//! Exercice du chapitre 5 : les entrées du clavier.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_05
//! ```

use bevy::prelude::*;

/// Marque le joueur.
#[derive(Component)]
pub struct Joueur;

/// Un projectile tiré par le joueur.
#[derive(Component)]
pub struct Projectile;

/// La vitesse de marche du joueur, en pixels par seconde.
pub const VITESSE_JOUEUR: f32 = 200.0;

/// Renvoie la direction choisie avec le clavier, de longueur 1, ou nulle si aucune direction
/// n'est choisie.
///
/// Les flèches et les touches W, A, S, D (Z, Q, S, D sur un clavier AZERTY) donnent la direction :
/// haut (`+y`), gauche (`-x`), bas (`-y`) et droite (`+x`). Deux touches opposées s'annulent.
///
/// Indices :
/// - `clavier.pressed(KeyCode::ArrowUp)` dit si une touche est enfoncée, et
///   `clavier.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW])` si au moins une des touches l'est ;
/// - `Vec2::normalize_or_zero` donne un vecteur de même direction et de longueur 1 (ou nul).
pub fn direction(clavier: &ButtonInput<KeyCode>) -> Vec2 {
    todo!()
}

/// Déplace le joueur dans la [`direction`] choisie, à [`VITESSE_JOUEUR`] pixels par seconde, ou
/// deux fois plus vite quand la touche Maj de gauche (`KeyCode::ShiftLeft`) est enfoncée.
///
/// Indice : `Res<ButtonInput<KeyCode>>` donne l'état du clavier.
pub fn deplacer_le_joueur(temps: Res<Time>) {
    todo!()
}

/// Quand la barre d'espace vient d'être enfoncée, crée un [`Projectile`] à la position du joueur.
///
/// Garder la touche enfoncée ne tire qu'un seul projectile.
///
/// Indice : `clavier.just_pressed(touche)` n'est vrai que pendant l'image où la touche a été
/// enfoncée.
pub fn tirer(mut commands: Commands) {
    todo!()
}

/// Le plugin du chapitre : le joueur au centre au démarrage, et les deux systèmes à chaque image.
///
/// Indice : un système de démarrage peut être une fonction ou une fermeture (*closure*) :
/// `app.add_systems(Startup, |mut commands: Commands| { ... })`.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
