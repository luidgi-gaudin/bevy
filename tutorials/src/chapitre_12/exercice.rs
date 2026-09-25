//! Exercice du chapitre 12 : les collisions avec la carte.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_12
//! ```
//!
//! Ce chapitre utilise `Physique` et `simuler_le_mouvement` de la solution du chapitre 11.

use bevy::prelude::*;
use bevy_tutorials::chapitre_11::solution::{simuler_le_mouvement, Physique};

/// La moitié de la largeur du joueur, en mètres : sa boîte de collision fait 0,8 m de côté.
pub const DEMI_LARGEUR_JOUEUR: f32 = 0.4;

/// La hauteur du joueur, en mètres.
pub const HAUTEUR_JOUEUR: f32 = 1.8;

/// En dessous de cette distance, en mètres, deux boîtes qui se touchent ne sont pas considérées
/// comme en collision : sans cette marge, les erreurs d'arrondi bloqueraient un joueur qui longe
/// un mur.
pub const MARGE: f32 = 1e-4;

/// Un obstacle de la carte (un mur, une caisse, une plateforme...) : une boîte alignée sur les
/// axes.
#[derive(Component, Debug, Clone, Copy)]
pub struct Obstacle(pub Aabb3d);

impl Obstacle {
    /// Un obstacle de centre `centre` et de dimensions `taille`, en mètres.
    pub fn new(centre: Vec3, taille: Vec3) -> Self {
        Self(Aabb3d::new(centre, taille / 2.0))
    }
}

/// Le résultat d'un déplacement avec collisions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deplacement {
    /// La position finale des pieds.
    pub position: Vec3,
    /// Pour chaque axe (X, Y, Z), le déplacement a-t-il été arrêté par un obstacle ?
    pub bloque: [bool; 3],
    /// Le joueur s'est-il posé sur un obstacle ?
    pub au_sol: bool,
}

/// Renvoie la boîte de collision d'un joueur dont les pieds sont en `position` : 0,8 m de large
/// et de profondeur, centrée sur les pieds, et [`HAUTEUR_JOUEUR`] de haut, au-dessus des pieds.
///
/// Indice : `Aabb3d { min, max }` attend des `Vec3A` : `vec3.into()` fait la conversion.
pub fn boite_du_joueur(position: Vec3) -> Aabb3d {
    todo!()
}

/// Renvoie `true` si deux boîtes se chevauchent de plus de [`MARGE`] sur chaque axe.
///
/// Indice : `a.cmplt(b).all()` dit si chaque coordonnée de `a` est inférieure à celle de `b`.
pub fn se_chevauchent(a: &Aabb3d, b: &Aabb3d) -> bool {
    todo!()
}

/// Déplace un joueur dont les pieds sont en `depart` de `deplacement`, axe par axe (X, puis Z,
/// puis Y), en l'arrêtant au contact des obstacles.
///
/// Pour chaque axe : on avance sur cet axe, puis, pour chaque obstacle chevauché, on recule
/// jusqu'au contact du côté d'où l'on vient, et l'axe est bloqué. Un joueur arrêté en descendant
/// (axe Y, déplacement négatif) s'est posé sur l'obstacle.
///
/// Indice : `position[axe]` et `obstacle.min[axe]` donnent une coordonnée (0 pour X, 1 pour Y, 2
/// pour Z).
pub fn deplacer_avec_collisions(
    depart: Vec3,
    deplacement: Vec3,
    obstacles: &[Aabb3d],
) -> Deplacement {
    todo!()
}

/// Corrige un tick de mouvement : refait le déplacement du tick (de la position précédente à la
/// position) avec les collisions, annule la vitesse sur les axes bloqués, et pose le joueur sur
/// les obstacles (sans oublier qu'il était peut-être déjà au sol).
pub fn corriger(physique: &Physique, obstacles: &[Aabb3d]) -> Physique {
    todo!()
}

/// Corrige le dernier tick de mouvement de chaque joueur, pour qu'aucun ne traverse les
/// obstacles. S'exécute dans `FixedUpdate`, juste après la simulation du mouvement.
pub fn resoudre_les_collisions() {
    todo!()
}

/// Le plugin du chapitre, à ajouter avec celui du chapitre 11 : [`resoudre_les_collisions`] dans
/// `FixedUpdate`, après `simuler_le_mouvement`.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
