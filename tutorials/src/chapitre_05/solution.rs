//! Solution du chapitre 5 : les entrées du clavier.

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
    // Sans cela, le joueur irait plus vite en diagonale : la longueur de (1, 1) est 1,41.
    direction.normalize_or_zero()
}

/// Déplace le joueur dans la [`direction`] choisie, à [`VITESSE_JOUEUR`] pixels par seconde, ou
/// deux fois plus vite quand la touche Maj de gauche est enfoncée.
pub fn deplacer_le_joueur(
    clavier: Res<ButtonInput<KeyCode>>,
    temps: Res<Time>,
    mut joueur: Single<&mut Transform, With<Joueur>>,
) {
    let mut vitesse = VITESSE_JOUEUR;
    if clavier.pressed(KeyCode::ShiftLeft) {
        vitesse *= 2.0;
    }
    let deplacement = direction(&clavier) * vitesse * temps.delta_secs();
    joueur.translation += deplacement.extend(0.0);
}

/// Quand la barre d'espace vient d'être enfoncée, crée un [`Projectile`] à la position du joueur.
///
/// Garder la touche enfoncée ne tire qu'un seul projectile.
pub fn tirer(
    mut commands: Commands,
    clavier: Res<ButtonInput<KeyCode>>,
    joueur: Single<&Transform, With<Joueur>>,
) {
    if clavier.just_pressed(KeyCode::Space) {
        commands.spawn((Projectile, Transform::from_translation(joueur.translation)));
    }
}

/// Le plugin du chapitre : le joueur au centre au démarrage, et les deux systèmes à chaque image.
pub fn plugin(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Joueur, Transform::default()));
    })
    .add_systems(Update, (deplacer_le_joueur, tirer));
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
