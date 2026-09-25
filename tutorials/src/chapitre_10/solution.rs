//! Solution du chapitre 10 : la visée à la souris d'un FPS.

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};
use core::f32::consts::{PI, TAU};

/// L'angle dont tourne le regard pour un point de souris, à une sensibilité de 1, en degrés.
///
/// C'est la valeur de Quake et de Counter-Strike (`m_yaw`) : les joueurs peuvent ainsi garder la
/// même sensibilité d'un jeu à l'autre.
pub const DEGRES_PAR_POINT: f32 = 0.022;

/// L'angle maximal pour lever ou baisser les yeux : un peu moins que la verticale, pour ne jamais
/// basculer « par-dessus ».
pub const TANGAGE_MAX: f32 = 89.0 * PI / 180.0;

/// Marque le joueur.
#[derive(Component, Default)]
pub struct Joueur;

/// L'orientation du regard, en radians.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Orientation {
    /// L'angle horizontal (tourner à gauche ou à droite, autour de l'axe vertical), entre -π et π.
    /// À 0, le regard est tourné vers -Z ; un angle positif tourne vers la gauche.
    pub lacet: f32,
    /// L'angle vertical (lever ou baisser les yeux), entre -[`TANGAGE_MAX`] et [`TANGAGE_MAX`].
    /// Un angle positif regarde vers le haut.
    pub tangage: f32,
}

/// La sensibilité de la souris : un point de souris tourne le regard de
/// [`DEGRES_PAR_POINT`] × sensibilité degrés.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Sensibilite(pub f32);

impl Default for Sensibilite {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Renvoie la distance, en centimètres, dont il faut déplacer la souris pour faire un tour
/// complet (360°), avec une souris de `dpi` points par pouce (2,54 cm).
///
/// C'est la façon la plus précise de comparer des sensibilités, d'une souris ou d'un jeu à
/// l'autre.
pub fn centimetres_par_tour(sensibilite: f32, dpi: f32) -> f32 {
    let points_par_tour = 360.0 / (DEGRES_PAR_POINT * sensibilite);
    points_par_tour / dpi * 2.54
}

/// Ramène un angle entre -π et π.
pub fn normaliser_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Renvoie la rotation qui correspond à une orientation.
pub fn rotation(orientation: &Orientation) -> Quat {
    Quat::from_euler(EulerRot::YXZ, orientation.lacet, orientation.tangage, 0.0)
}

/// Renvoie la direction du regard : un vecteur de longueur 1.
pub fn direction_du_regard(orientation: &Orientation) -> Vec3 {
    rotation(orientation) * Vec3::NEG_Z
}

/// Tourne le regard du joueur selon le mouvement de la souris pendant l'image.
///
/// La souris vers la droite tourne le regard à droite, la souris vers le haut (un déplacement
/// vertical négatif, comme à l'écran) fait lever les yeux. Le mouvement de la souris n'est
/// **pas** multiplié par la durée de l'image : c'est une distance, pas une vitesse.
pub fn viser(
    souris: Res<AccumulatedMouseMotion>,
    sensibilite: Res<Sensibilite>,
    mut orientations: Query<&mut Orientation, With<Joueur>>,
) {
    let angle_par_point = (DEGRES_PAR_POINT * sensibilite.0).to_radians();
    for mut orientation in &mut orientations {
        orientation.lacet = normaliser_angle(orientation.lacet - souris.delta.x * angle_par_point);
        orientation.tangage = (orientation.tangage - souris.delta.y * angle_par_point)
            .clamp(-TANGAGE_MAX, TANGAGE_MAX);
    }
}

/// Tourne le `Transform` des entités selon leur [`Orientation`].
pub fn orienter(mut entites: Query<(&Orientation, &mut Transform)>) {
    for (orientation, mut transform) in &mut entites {
        transform.rotation = rotation(orientation);
    }
}

/// Le plugin du chapitre.
pub fn plugin(app: &mut App) {
    app.init_resource::<Sensibilite>()
        // Normalement ajoutée par le `InputPlugin` : ne fait rien si elle existe déjà.
        .init_resource::<AccumulatedMouseMotion>()
        .add_systems(Update, (viser, orienter).chain());
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
