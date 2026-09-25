//! Solution du chapitre 4 : le temps, les minuteurs et les déplacements réguliers.

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
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

/// Le temps écoulé depuis le lancement du jeu, en secondes.
#[derive(Resource, Default)]
pub struct Chrono(pub f32);

/// La durée de vie des pièces, en secondes.
pub const DUREE_DE_VIE_PIECES: f32 = 5.0;

/// Déplace chaque entité selon sa [`Vitesse`], en tenant compte de la durée de l'image.
pub fn deplacer(temps: Res<Time>, mut entites: Query<(&mut Transform, &Vitesse)>) {
    for (mut transform, vitesse) in &mut entites {
        transform.translation += (vitesse.0 * temps.delta_secs()).extend(0.0);
    }
}

/// Fait apparaître une pièce toutes les 2 secondes, au centre, avec une durée de vie de
/// [`DUREE_DE_VIE_PIECES`] secondes.
pub fn faire_apparaitre_des_pieces(
    mut commands: Commands,
    temps: Res<Time>,
    mut minuteur: ResMut<MinuteurPieces>,
) {
    minuteur.0.tick(temps.delta());
    if minuteur.0.just_finished() {
        commands.spawn((
            Piece,
            Transform::default(),
            DureeDeVie(Timer::from_seconds(DUREE_DE_VIE_PIECES, TimerMode::Once)),
        ));
    }
}

/// Fait avancer la [`DureeDeVie`] des entités, et détruit celles dont la durée de vie est
/// terminée.
pub fn faire_disparaitre(
    mut commands: Commands,
    temps: Res<Time>,
    mut entites: Query<(Entity, &mut DureeDeVie)>,
) {
    for (entite, mut duree_de_vie) in &mut entites {
        if duree_de_vie.0.tick(temps.delta()).is_finished() {
            commands.entity(entite).despawn();
        }
    }
}

/// Écrit le temps écoulé depuis le lancement du jeu dans le [`Chrono`].
pub fn mettre_a_jour_le_chrono(temps: Res<Time>, mut chrono: ResMut<Chrono>) {
    chrono.0 = temps.elapsed_secs();
}

/// Le plugin du chapitre.
pub fn plugin(app: &mut App) {
    app.init_resource::<MinuteurPieces>()
        .init_resource::<Chrono>()
        .add_systems(
            Update,
            (
                deplacer,
                faire_apparaitre_des_pieces,
                faire_disparaitre,
                mettre_a_jour_le_chrono,
            ),
        );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
