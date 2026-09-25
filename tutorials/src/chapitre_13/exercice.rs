//! Exercice du chapitre 13 : les armes hitscan, les hitbox et les dégâts.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_13
//! ```
//!
//! Ce chapitre utilise les solutions des chapitres 10 à 12.

use bevy::prelude::*;
use bevy_tutorials::chapitre_10::solution::{direction_du_regard, Joueur, Orientation};
use bevy_tutorials::chapitre_11::solution::{Commandes, Physique};
use bevy_tutorials::chapitre_12::solution::{resoudre_les_collisions, Obstacle};

/// La hauteur des yeux au-dessus des pieds, en mètres : les tirs partent de là.
pub const HAUTEUR_DES_YEUX: f32 = 1.6;

/// La portée maximale d'un tir, en mètres.
pub const PORTEE_MAX: f32 = 250.0;

/// Une zone du corps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    /// La tête : les dégâts sont doublés.
    Tete,
    /// Le torse : les dégâts de base.
    Torse,
    /// Les jambes : les dégâts sont réduits.
    Jambes,
}

impl Zone {
    /// Le multiplicateur de dégâts de la zone : 2 pour la tête, 1 pour le torse, 0,8 pour les
    /// jambes.
    pub fn multiplicateur(self) -> f32 {
        todo!()
    }
}

/// Une boîte qui peut être touchée par les tirs, placée par rapport aux pieds du personnage.
#[derive(Debug, Clone, Copy)]
pub struct Hitbox {
    /// La zone du corps.
    pub zone: Zone,
    /// La boîte, par rapport aux pieds.
    pub boite: Aabb3d,
}

/// Les hitbox d'un personnage.
#[derive(Component, Debug, Clone)]
pub struct Hitboxes(pub Vec<Hitbox>);

impl Default for Hitboxes {
    /// Les hitbox d'un personnage de 1,8 m : les jambes, le torse et la tête.
    fn default() -> Self {
        let hitbox = |zone, centre, demi_taille| Hitbox {
            zone,
            boite: Aabb3d::new(centre, demi_taille),
        };
        Self(vec![
            hitbox(
                Zone::Jambes,
                Vec3::new(0.0, 0.45, 0.0),
                Vec3::new(0.3, 0.45, 0.2),
            ),
            hitbox(
                Zone::Torse,
                Vec3::new(0.0, 1.2, 0.0),
                Vec3::new(0.35, 0.3, 0.2),
            ),
            hitbox(
                Zone::Tete,
                Vec3::new(0.0, 1.65, 0.0),
                Vec3::new(0.12, 0.15, 0.12),
            ),
        ])
    }
}

/// Les caractéristiques d'une arme.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Arme {
    /// Les dégâts d'une balle dans le torse, à courte portée.
    pub degats: f32,
    /// La cadence de tir, en tirs par minute.
    pub cadence: f32,
    /// Le nombre de balles d'un chargeur plein.
    pub taille_chargeur: u32,
    /// La durée d'un rechargement, en secondes.
    pub duree_rechargement: f32,
    /// Jusqu'à cette distance, en mètres, les dégâts sont maximaux.
    pub portee_efficace: f32,
    /// La part des dégâts qui reste au double de la portée efficace et au-delà.
    pub degats_a_longue_portee: f32,
}

impl Arme {
    /// Un fusil d'assaut : 30 dégâts, 600 tirs par minute, 30 balles.
    pub fn fusil_d_assaut() -> Self {
        Self {
            degats: 30.0,
            cadence: 600.0,
            taille_chargeur: 30,
            duree_rechargement: 2.0,
            portee_efficace: 25.0,
            degats_a_longue_portee: 0.6,
        }
    }
}

/// L'état d'une arme.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct EtatArme {
    /// Les balles restantes dans le chargeur.
    pub munitions: u32,
    /// Le temps avant de pouvoir tirer à nouveau, en secondes.
    pub attente: f32,
    /// Le temps restant avant la fin du rechargement, s'il est en cours.
    pub rechargement: Option<f32>,
    /// Le nombre de balles tirées depuis le début de la partie.
    pub balles_tirees: u32,
}

impl EtatArme {
    /// Une arme prête à tirer, chargeur plein.
    pub fn charge(arme: &Arme) -> Self {
        Self {
            munitions: arme.taille_chargeur,
            attente: 0.0,
            rechargement: None,
            balles_tirees: 0,
        }
    }
}

/// Les commandes de tir : lues à chaque image, utilisées à chaque tick.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct CommandesDeTir {
    /// La gâchette est enfoncée.
    pub tir: bool,
    /// Un rechargement demandé, en attente du prochain tick.
    pub recharger: bool,
}

/// Envoyé pour chaque balle tirée : pour les effets, les sons, le recul...
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Tir {
    /// Le personnage qui a tiré.
    pub tireur: Entity,
    /// Le point de départ de la balle.
    pub origine: Vec3,
    /// La direction de la balle.
    pub direction: Vec3,
}

/// Envoyé quand une balle touche un personnage.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Touche {
    /// Le personnage qui a tiré.
    pub tireur: Entity,
    /// Le personnage touché.
    pub cible: Entity,
    /// La zone touchée.
    pub zone: Zone,
    /// La distance du tir, en mètres.
    pub distance: f32,
    /// Les dégâts infligés.
    pub degats: f32,
}

/// Ce qu'une balle touche en premier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Impact {
    /// Un obstacle de la carte, à cette distance.
    Obstacle {
        /// La distance de l'impact, en mètres.
        distance: f32,
    },
    /// Un personnage.
    Personnage {
        /// Le personnage touché.
        entite: Entity,
        /// La zone touchée.
        zone: Zone,
        /// La distance de l'impact, en mètres.
        distance: f32,
    },
}

impl Impact {
    /// La distance de l'impact, en mètres.
    pub fn distance(&self) -> f32 {
        match *self {
            Impact::Obstacle { distance } | Impact::Personnage { distance, .. } => distance,
        }
    }
}

/// Lance une balle depuis `origine` dans la `direction`, et renvoie ce qu'elle touche en premier
/// : un obstacle, ou une hitbox d'un des `personnages` (dont on donne la position des pieds).
///
/// Le personnage `tireur` est ignoré : on ne se tire pas dessus.
///
/// Indices :
/// - `RayCast3d::new(origine, direction, PORTEE_MAX)` crée un rayon, et
///   `rayon.aabb_intersection_at(&boite)` renvoie la distance à laquelle il touche une boîte ;
/// - les hitbox sont placées par rapport aux pieds : ajoutez la position des pieds (convertie en
///   `Vec3A`) à `min` et `max`.
pub fn premier_impact<'a>(
    origine: Vec3,
    direction: Dir3,
    obstacles: &[Aabb3d],
    personnages: impl IntoIterator<Item = (Entity, Vec3, &'a Hitboxes)>,
    tireur: Entity,
) -> Option<Impact> {
    todo!()
}

/// Renvoie les dégâts d'une balle de l'`arme`, selon la zone touchée et la distance.
///
/// Jusqu'à la portée efficace, les dégâts sont maximaux ; ils diminuent ensuite linéairement
/// jusqu'à [`Arme::degats_a_longue_portee`], atteints au double de la portée efficace.
pub fn calculer_les_degats(arme: &Arme, zone: Zone, distance: f32) -> f32 {
    todo!()
}

/// Fait avancer l'état de l'arme d'un tick de `dt` secondes, et renvoie `true` si une balle est
/// tirée pendant ce tick.
///
/// - Un rechargement en cours empêche de tirer ; à sa fin, le chargeur est plein.
/// - Un rechargement demandé commence si le chargeur n'est pas plein.
/// - Tirer avec un chargeur vide lance le rechargement.
/// - Entre deux balles, il faut attendre `60 / cadence` secondes. Pour respecter exactement la
///   cadence, le temps d'attente en trop est reporté sur la balle suivante, mais seulement tant
///   que la gâchette reste enfoncée : sinon, on pourrait accumuler du temps pour tirer une rafale.
pub fn actionner_l_arme(
    etat: &mut EtatArme,
    arme: &Arme,
    veut_tirer: bool,
    veut_recharger: bool,
    dt: f32,
) -> bool {
    todo!()
}

/// Lit la souris et le clavier, et met à jour les [`CommandesDeTir`] du joueur : le clic gauche
/// tire, la touche R recharge (la demande reste en attente jusqu'au prochain tick).
///
/// Indice : `Res<ButtonInput<MouseButton>>` fonctionne comme le clavier.
pub fn lire_les_commandes_de_tir() {
    todo!()
}

/// Fait tirer les personnages armés, à chaque tick : une balle part des yeux, dans la direction
/// du regard. On ne peut pas tirer en sprintant (en avançant).
///
/// Envoie un message [`Tir`] pour chaque balle, et un message [`Touche`] quand elle touche un
/// personnage. Consomme les demandes de rechargement.
pub fn tirer(temps: Res<Time>) {
    todo!()
}

/// Le plugin du chapitre, à ajouter avec ceux des chapitres 11 et 12 : les messages, la lecture
/// des commandes avant les ticks, et [`tirer`] dans `FixedUpdate`, après les collisions.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
