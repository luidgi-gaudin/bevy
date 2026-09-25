//! Solution du chapitre 16 : des bots pour s'entraîner.

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
pub fn orientation_vers(depuis: Vec3, vers: Vec3) -> Orientation {
    let direction = vers - depuis;
    let horizontale = Vec2::new(direction.x, direction.z).length();
    Orientation {
        lacet: ops::atan2(-direction.x, -direction.z),
        tangage: ops::atan2(direction.y, horizontale).clamp(-TANGAGE_MAX, TANGAGE_MAX),
    }
}

/// Tourne l'orientation `actuelle` vers l'orientation `voulue`, d'au plus `angle_max` radians sur
/// chaque axe, par le chemin le plus court.
pub fn tourner_vers(actuelle: Orientation, voulue: Orientation, angle_max: f32) -> Orientation {
    let ecart_lacet = normaliser_angle(voulue.lacet - actuelle.lacet);
    let ecart_tangage = voulue.tangage - actuelle.tangage;
    Orientation {
        lacet: normaliser_angle(actuelle.lacet + ecart_lacet.clamp(-angle_max, angle_max)),
        tangage: actuelle.tangage + ecart_tangage.clamp(-angle_max, angle_max),
    }
}

/// Renvoie `true` si aucun obstacle ne se trouve entre `depuis` et `vers`.
pub fn ligne_de_vue(depuis: Vec3, vers: Vec3, obstacles: &[Aabb3d]) -> bool {
    let Ok((direction, distance)) = Dir3::new_and_length(vers - depuis) else {
        return true;
    };
    let rayon = RayCast3d::new(depuis, direction, distance);
    obstacles
        .iter()
        .all(|obstacle| rayon.aabb_intersection_at(obstacle).is_none())
}

/// Le cerveau des bots, à chaque tick : choisir une cible visible, la viser, tirer après le temps
/// de réaction en se décalant sur le côté ; sans cible, patrouiller entre les points
/// d'apparition.
pub fn penser(
    temps: Res<Time>,
    obstacles: Query<&Obstacle>,
    points: Query<&PointDApparition>,
    mut bots: Query<(
        Entity,
        &mut Bot,
        &Physique,
        &mut Orientation,
        &mut Commandes,
        &mut CommandesDeTir,
    )>,
    ennemis: Query<(Entity, &Physique), (With<Combattant>, With<Hitboxes>, Without<Mort>)>,
) {
    let dt = temps.delta_secs();
    let obstacles: Vec<Aabb3d> = obstacles.iter().map(|obstacle| obstacle.0).collect();
    let points: Vec<Vec3> = points.iter().map(|point| point.0).collect();

    for (entite, mut bot, physique, mut orientation, mut commandes, mut commandes_de_tir) in
        &mut bots
    {
        let yeux = physique.position + Vec3::Y * HAUTEUR_DES_YEUX;
        let hauteur_visee = bot.niveau.hauteur_visee;
        let visee = |pieds: Vec3| pieds + Vec3::Y * hauteur_visee;

        // La cible : l'ennemi visible le plus proche.
        let cible = ennemis
            .iter()
            .filter(|&(ennemi, physique)| {
                ennemi != entite && ligne_de_vue(yeux, visee(physique.position), &obstacles)
            })
            .min_by(|(_, a), (_, b)| {
                a.position
                    .distance(yeux)
                    .total_cmp(&b.position.distance(yeux))
            })
            .map(|(ennemi, physique)| (ennemi, visee(physique.position)));
        let cible_actuelle = cible.map(|(ennemi, _)| ennemi);
        if cible_actuelle != bot.cible {
            bot.cible = cible_actuelle;
            bot.reaction = bot.niveau.temps_de_reaction;
        }

        let rotation_max = bot.niveau.vitesse_de_rotation.to_radians() * dt;
        if let Some((_, point_vise)) = cible {
            // Viser, et tirer une fois le temps de réaction écoulé.
            *orientation = tourner_vers(
                *orientation,
                orientation_vers(yeux, point_vise),
                rotation_max,
            );
            bot.reaction -= dt;
            let ecart = direction_du_regard(&orientation)
                .angle_between(point_vise - yeux)
                .to_degrees();
            commandes_de_tir.tir = bot.reaction <= 0.0 && ecart <= bot.niveau.tolerance;

            // Se décaler de gauche à droite, pour être plus difficile à toucher.
            bot.pas_de_cote -= dt;
            if bot.pas_de_cote <= 0.0 {
                bot.pas_de_cote += DUREE_D_UN_PAS_DE_COTE;
                bot.cote = -bot.cote;
            }
            commandes.avant = 0.0;
            commandes.droite = bot.cote;
        } else {
            // Patrouiller : marcher vers le point de passage, en regardant devant soi.
            commandes_de_tir.tir = false;
            commandes.droite = 0.0;
            commandes.avant = 0.0;
            if points.is_empty() {
                continue;
            }
            let mut destination = points[bot.point_de_passage % points.len()];
            if destination.distance(physique.position) < DISTANCE_D_ARRIVEE {
                bot.point_de_passage = (bot.point_de_passage + 1) % points.len();
                destination = points[bot.point_de_passage];
            }
            let voulue = Orientation {
                tangage: 0.0,
                ..orientation_vers(physique.position, destination)
            };
            *orientation = tourner_vers(*orientation, voulue, rotation_max);
            commandes.avant = 1.0;
        }
    }
}

/// Le plugin du chapitre : les bots pensent à chaque tick, avant la simulation du mouvement (et
/// donc avant le tir).
pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, penser.before(simuler_le_mouvement));
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
