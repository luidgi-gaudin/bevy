//! Exercice du chapitre 10 : la visée à la souris d'un FPS.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_10
//! ```

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
pub fn centimetres_par_tour(sensibilite: f32, dpi: f32) -> f32 {
    todo!()
}

/// Ramène un angle entre -π et π.
///
/// Indice : `f32::rem_euclid(TAU)` ramène un angle entre 0 et 2π.
pub fn normaliser_angle(angle: f32) -> f32 {
    todo!()
}

/// Renvoie la rotation qui correspond à une orientation.
///
/// Indice : `Quat::from_euler(EulerRot::YXZ, lacet, tangage, 0.0)` tourne d'abord autour de
/// l'axe vertical (Y), puis autour de l'axe horizontal (X).
pub fn rotation(orientation: &Orientation) -> Quat {
    todo!()
}

/// Renvoie la direction du regard : un vecteur de longueur 1.
///
/// Indice : sans rotation, on regarde vers `Vec3::NEG_Z`, et `rotation * vecteur` tourne un
/// vecteur.
pub fn direction_du_regard(orientation: &Orientation) -> Vec3 {
    todo!()
}

/// Tourne le regard du joueur selon le mouvement de la souris pendant l'image.
///
/// La souris vers la droite tourne le regard à droite, la souris vers le haut (un déplacement
/// vertical négatif, comme à l'écran) fait lever les yeux. Le lacet reste entre -π et π, et le
/// tangage entre -[`TANGAGE_MAX`] et [`TANGAGE_MAX`].
///
/// Indice : `Res<AccumulatedMouseMotion>` donne le déplacement de la souris pendant l'image, dans
/// son champ `delta`.
pub fn viser(sensibilite: Res<Sensibilite>) {
    todo!()
}

/// Tourne le `Transform` des entités selon leur [`Orientation`].
pub fn orienter() {
    todo!()
}

/// Le plugin du chapitre : les ressources [`Sensibilite`] et `AccumulatedMouseMotion`, puis
/// [`viser`] et [`orienter`] à chaque image, dans cet ordre.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
