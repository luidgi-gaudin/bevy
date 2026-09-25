//! Exercice du chapitre 14 : le maniement des armes, façon Call of Duty : la visée, le recul et
//! la dispersion.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_14
//! ```
//!
//! Ce chapitre utilise les solutions des chapitres 10 à 13.

use bevy::{math::ops, prelude::*};
use bevy_tutorials::chapitre_10::solution::{
    direction_du_regard, normaliser_angle, Joueur, Orientation, TANGAGE_MAX,
};
use bevy_tutorials::chapitre_11::solution::{lire_le_clavier, Commandes, Physique, VITESSE_MARCHE};
use bevy_tutorials::chapitre_12::solution::{resoudre_les_collisions, Obstacle};
use bevy_tutorials::chapitre_13::solution::{
    actionner_l_arme, calculer_les_degats, lire_les_commandes_de_tir, premier_impact, Arme,
    CommandesDeTir, EtatArme, Hitboxes, Impact, Tir, Touche, HAUTEUR_DES_YEUX,
};
use core::f32::consts::TAU;

/// Le champ de vision vertical à la hanche, en degrés.
pub const CHAMP_DE_VISION: f32 = 70.0;

/// Le champ de vision vertical en visant, en degrés : l'image est agrandie.
pub const CHAMP_DE_VISION_EN_VISANT: f32 = 45.0;

/// Le temps pour épauler l'arme, en secondes.
pub const DUREE_POUR_EPAULER: f32 = 0.2;

/// En visant, la vitesse de déplacement est multipliée par ce nombre.
pub const RALENTISSEMENT_EN_VISANT: f32 = 0.6;

/// La dispersion des tirs à la hanche, en degrés : l'angle maximal entre le regard et la balle.
pub const DISPERSION_A_LA_HANCHE: f32 = 3.0;

/// La dispersion des tirs en visant, en degrés.
pub const DISPERSION_EN_VISANT: f32 = 0.3;

/// La dispersion ajoutée en l'air, en degrés.
pub const DISPERSION_EN_L_AIR: f32 = 5.0;

/// Après ce temps sans tirer, en secondes, le recul recommence au début de son motif.
pub const DELAI_DE_RETOUR_DU_RECUL: f32 = 0.3;

/// La visée : épauler l'arme pour viser dans le viseur (en anglais *ADS*, pour *aim down
/// sights*).
#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Visee {
    /// Le joueur veut viser (bouton droit de la souris enfoncé).
    pub commande: bool,
    /// De 0 (l'arme à la hanche) à 1 (l'arme épaulée).
    pub progression: f32,
}

/// Le recul d'une arme : un motif fixe, que les joueurs apprennent à compenser.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Recul {
    /// Le mouvement du regard à chaque balle, en degrés : `x` vers la droite, `y` vers le haut.
    /// Après la dernière balle du motif, la dernière valeur se répète.
    pub motif: Vec<Vec2>,
    /// Le numéro de la prochaine balle dans le motif.
    pub balle: usize,
    /// Le temps écoulé depuis la dernière balle, en secondes.
    pub repos: f32,
}

impl Recul {
    /// Le recul d'un fusil d'assaut : il monte, puis part à gauche et à droite.
    pub fn fusil_d_assaut() -> Self {
        let motif = [
            (0.0, 0.9),
            (0.1, 1.0),
            (-0.1, 1.0),
            (0.15, 0.9),
            (0.25, 0.8),
            (-0.15, 0.7),
            (-0.35, 0.6),
            (-0.3, 0.5),
            (0.2, 0.5),
            (0.4, 0.4),
            (0.3, 0.4),
            (-0.2, 0.4),
        ];
        Self {
            motif: motif.into_iter().map(|(x, y)| Vec2::new(x, y)).collect(),
            balle: 0,
            repos: 0.0,
        }
    }
}

/// Renvoie le champ de vision vertical, en radians, selon la progression de la visée.
///
/// Indice : `a.lerp(b, t)` interpole aussi entre deux `f32`.
pub fn champ_de_vision(progression: f32) -> f32 {
    todo!()
}

/// Fait progresser la visée pendant `dt` secondes : vers 1 si le joueur vise, vers 0 sinon. Il
/// faut [`DUREE_POUR_EPAULER`] secondes pour passer de l'un à l'autre.
pub fn epauler(visee: &mut Visee, dt: f32) {
    todo!()
}

/// Renvoie la dispersion d'un tir, en degrés, selon la visée et le mouvement du tireur.
///
/// La dispersion de base va de [`DISPERSION_A_LA_HANCHE`] à [`DISPERSION_EN_VISANT`] selon la
/// visée. Elle est multipliée par `1 + vitesse / VITESSE_MARCHE` (doublée en marchant ; seule
/// la vitesse horizontale compte), et augmentée de [`DISPERSION_EN_L_AIR`] en l'air.
pub fn dispersion(progression: f32, physique: &Physique) -> f32 {
    todo!()
}

/// Renvoie un nombre qui semble aléatoire, entre 0 (inclus) et 1 (exclus), mais qui est toujours
/// le même pour une même graine.
pub fn hasard(graine: u32) -> f32 {
    // Une fonction de hachage (PCG) : de petits changements de la graine changent tout le
    // résultat.
    let etat = graine.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let melange = ((etat >> ((etat >> 28) + 4)) ^ etat).wrapping_mul(277_803_737);
    let melange = (melange >> 22) ^ melange;
    (melange >> 8) as f32 / (1 << 24) as f32
}

/// Renvoie la graine d'une balle : différente pour chaque tireur et chaque balle, mais identique
/// sur tous les ordinateurs qui simulent la partie.
pub fn graine_de_balle(tireur: Entity, balle: u32) -> u32 {
    let bits = tireur.to_bits();
    ((bits as u32) ^ ((bits >> 32) as u32)).wrapping_mul(0x9E37_79B9)
        ^ balle.wrapping_mul(0x85EB_CA6B)
}

/// Dévie une `direction` (de longueur 1) d'un angle au hasard, dans un cône de `dispersion`
/// degrés. La même graine donne toujours la même déviation.
///
/// Indices :
/// - l'angle de déviation est `hasard(graine).sqrt() × dispersion` (la racine carrée répartit
///   les balles uniformément dans le cône, au lieu de les concentrer au centre) ;
/// - la direction de la déviation autour du regard est `hasard(autre_graine) × TAU` ;
/// - `direction.any_orthonormal_pair()` donne deux axes perpendiculaires au regard.
pub fn devier(direction: Vec3, dispersion: f32, graine: u32) -> Vec3 {
    todo!()
}

/// Lit la souris et met à jour la commande de [`Visee`] du joueur : le bouton droit vise.
pub fn lire_la_visee() {
    todo!()
}

/// En visant, ralentit les déplacements (selon la progression de la visée) et empêche de
/// sprinter : à exécuter après la lecture du clavier.
pub fn ralentir_en_visant() {
    todo!()
}

/// Fait progresser la visée de chaque tireur, à chaque tick.
pub fn epauler_les_armes(temps: Res<Time>) {
    todo!()
}

/// Comme le système `tirer` du chapitre 13, mais chaque balle est déviée selon la
/// [`dispersion`], avec la graine de la balle (`graine_de_balle(tireur, etat.balles_tirees)`,
/// après le tir).
pub fn tirer_avec_dispersion(temps: Res<Time>) {
    todo!()
}

/// Applique le recul de chaque balle tirée au regard du tireur : le tangage monte de `y` degrés,
/// et le lacet tourne de `x` degrés vers la droite. En visant, le recul est réduit de moitié.
/// Après [`DELAI_DE_RETOUR_DU_RECUL`] secondes sans tirer, le motif recommence.
///
/// Indice : lisez les messages [`Tir`] avec un `MessageReader<Tir>`.
pub fn appliquer_le_recul(temps: Res<Time>) {
    todo!()
}

/// Le plugin du chapitre. Il remplace celui du chapitre 13 : les messages, la lecture des
/// commandes (tir, visée, puis ralentissement) avant les ticks, et dans `FixedUpdate`, après les
/// collisions : épauler, tirer, puis le recul.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
