//! Exercice du chapitre 16 : des bots pour s'entraîner.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_16
//! ```
//!
//! Ce chapitre utilise les solutions des chapitres 10 à 15.

use bevy::{math::ops, prelude::*};
use bevy_tutorials::chapitre_10::solution::{
    direction_du_regard, normaliser_angle, Orientation, TANGAGE_MAX,
};
use bevy_tutorials::chapitre_11::solution::{simuler_le_mouvement, Commandes, Physique};
use bevy_tutorials::chapitre_12::solution::Obstacle;
use bevy_tutorials::chapitre_13::solution::{CommandesDeTir, Hitboxes, HAUTEUR_DES_YEUX};
use bevy_tutorials::chapitre_15::solution::{Combattant, Mort, PointDApparition};

/// À cette distance d'un point de passage, en mètres, un bot passe au suivant.
pub const DISTANCE_D_ARRIVEE: f32 = 1.5;

/// Le temps entre deux changements de direction quand un bot se décale en combattant, en
/// secondes.
pub const DUREE_D_UN_PAS_DE_COTE: f32 = 1.2;

/// Le niveau d'un bot : ce qui le rend facile ou difficile à battre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Niveau {
    /// Le temps entre le moment où un ennemi devient visible et le premier tir, en secondes.
    pub temps_de_reaction: f32,
    /// La vitesse à laquelle le bot tourne son regard, en degrés par seconde.
    pub vitesse_de_rotation: f32,
    /// Le bot tire quand son regard est à moins de cet angle de sa cible, en degrés.
    pub tolerance: f32,
    /// La hauteur visée au-dessus des pieds de la cible, en mètres : 1,65 pour la tête, 1,2 pour
    /// le torse.
    pub hauteur_visee: f32,
}

impl Niveau {
    /// Un bot lent, qui vise le torse.
    pub const FACILE: Self = Self {
        temps_de_reaction: 0.6,
        vitesse_de_rotation: 90.0,
        tolerance: 4.0,
        hauteur_visee: 1.2,
    };
    /// Un bot vif, qui vise la tête.
    pub const DIFFICILE: Self = Self {
        temps_de_reaction: 0.25,
        vitesse_de_rotation: 360.0,
        tolerance: 1.0,
        hauteur_visee: 1.65,
    };
}

/// Un bot : un combattant contrôlé par l'ordinateur.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Bot {
    /// Le niveau du bot.
    pub niveau: Niveau,
    /// L'ennemi que le bot combat, s'il en voit un.
    pub cible: Option<Entity>,
    /// Le temps restant avant de pouvoir tirer sur la cible, en secondes.
    pub reaction: f32,
    /// Le numéro du prochain point de passage de la patrouille.
    pub point_de_passage: usize,
    /// Le temps restant avant de changer de côté en combattant, en secondes.
    pub pas_de_cote: f32,
    /// Le côté vers lequel le bot se décale : 1 à droite, -1 à gauche.
    pub cote: f32,
}

impl Bot {
    /// Un bot du niveau donné.
    pub fn new(niveau: Niveau) -> Self {
        Self {
            niveau,
            cible: None,
            reaction: 0.0,
            point_de_passage: 0,
            pas_de_cote: DUREE_D_UN_PAS_DE_COTE,
            cote: 1.0,
        }
    }
}

/// Renvoie l'orientation qui regarde de `depuis` vers `vers`.
///
/// Indices : avec `d = vers - depuis`, le lacet est `ops::atan2(-d.x, -d.z)`, et le tangage est
/// l'angle entre `d` et l'horizontale : `ops::atan2(d.y, longueur horizontale de d)`.
pub fn orientation_vers(depuis: Vec3, vers: Vec3) -> Orientation {
    todo!()
}

/// Tourne l'orientation `actuelle` vers l'orientation `voulue`, d'au plus `angle_max` radians sur
/// chaque axe, par le chemin le plus court.
pub fn tourner_vers(actuelle: Orientation, voulue: Orientation, angle_max: f32) -> Orientation {
    todo!()
}

/// Renvoie `true` si aucun obstacle ne se trouve entre `depuis` et `vers`.
///
/// Indice : `Dir3::new_and_length(vecteur)` donne la direction et la longueur d'un vecteur, et un
/// `RayCast3d` s'arrête à la distance maximale qu'on lui donne.
pub fn ligne_de_vue(depuis: Vec3, vers: Vec3, obstacles: &[Aabb3d]) -> bool {
    todo!()
}

/// Le cerveau des bots, à chaque tick :
///
/// 1. La cible est l'ennemi le plus proche (un `Combattant` avec des `Hitboxes`, pas `Mort`, et
///    pas le bot lui-même) dont le point visé (ses pieds + `hauteur_visee`) est en ligne de vue
///    des yeux du bot.
/// 2. Quand la cible change, le temps de réaction repart de `niveau.temps_de_reaction`.
/// 3. Avec une cible : tourner le regard vers le point visé (au plus `vitesse_de_rotation` degrés
///    par seconde), décompter le temps de réaction, tirer si le temps de réaction est écoulé et
///    que l'écart entre le regard et le point visé est sous la `tolerance`. Et se décaler sur le
///    côté (`droite` = `cote`, sans avancer), en changeant de côté toutes les
///    [`DUREE_D_UN_PAS_DE_COTE`] secondes.
/// 4. Sans cible : ne pas tirer, et marcher vers le point d'apparition numéro `point_de_passage`
///    (en tournant l'horizontale du regard vers lui) ; à moins de [`DISTANCE_D_ARRIVEE`], passer
///    au point suivant.
pub fn penser(temps: Res<Time>) {
    todo!()
}

/// Le plugin du chapitre : [`penser`] dans `FixedUpdate`, avant `simuler_le_mouvement` (et donc
/// avant le tir).
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
