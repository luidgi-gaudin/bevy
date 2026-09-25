//! Solution du chapitre 12 : les collisions avec la carte.

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

/// Renvoie la boîte de collision d'un joueur dont les pieds sont en `position`.
pub fn boite_du_joueur(position: Vec3) -> Aabb3d {
    let demi_largeur = Vec3::new(DEMI_LARGEUR_JOUEUR, 0.0, DEMI_LARGEUR_JOUEUR);
    Aabb3d {
        min: (position - demi_largeur).into(),
        max: (position + demi_largeur + Vec3::Y * HAUTEUR_JOUEUR).into(),
    }
}

/// Renvoie `true` si deux boîtes se chevauchent de plus de [`MARGE`] sur chaque axe.
pub fn se_chevauchent(a: &Aabb3d, b: &Aabb3d) -> bool {
    (a.min + MARGE).cmplt(b.max).all() && (a.max - MARGE).cmpgt(b.min).all()
}

/// Déplace un joueur dont les pieds sont en `depart` de `deplacement`, axe par axe (X, puis Z,
/// puis Y), en l'arrêtant au contact des obstacles.
///
/// Traiter les axes un par un permet de glisser le long des murs : un déplacement en diagonale
/// contre un mur est arrêté sur un axe, mais continue sur l'autre.
pub fn deplacer_avec_collisions(
    depart: Vec3,
    deplacement: Vec3,
    obstacles: &[Aabb3d],
) -> Deplacement {
    let mut position = depart;
    let mut bloque = [false; 3];
    let mut au_sol = false;
    // L'écart entre les pieds et les faces de la boîte du joueur.
    let vers_le_min = Vec3::new(-DEMI_LARGEUR_JOUEUR, 0.0, -DEMI_LARGEUR_JOUEUR);
    let vers_le_max = Vec3::new(DEMI_LARGEUR_JOUEUR, HAUTEUR_JOUEUR, DEMI_LARGEUR_JOUEUR);

    for axe in [0, 2, 1] {
        if deplacement[axe] == 0.0 {
            continue;
        }
        position[axe] += deplacement[axe];
        for obstacle in obstacles {
            if !se_chevauchent(&boite_du_joueur(position), obstacle) {
                continue;
            }
            // On recule jusqu'au contact, du côté d'où l'on vient.
            if deplacement[axe] > 0.0 {
                position[axe] = obstacle.min[axe] - vers_le_max[axe];
            } else {
                position[axe] = obstacle.max[axe] - vers_le_min[axe];
                if axe == 1 {
                    au_sol = true;
                }
            }
            bloque[axe] = true;
        }
    }
    Deplacement {
        position,
        bloque,
        au_sol,
    }
}

/// Corrige un tick de mouvement : refait le déplacement du tick avec les collisions, annule la
/// vitesse sur les axes bloqués, et pose le joueur sur les obstacles.
pub fn corriger(physique: &Physique, obstacles: &[Aabb3d]) -> Physique {
    let deplacement = physique.position - physique.position_precedente;
    let resultat = deplacer_avec_collisions(physique.position_precedente, deplacement, obstacles);
    let mut corrige = *physique;
    corrige.position = resultat.position;
    for axe in 0..3 {
        if resultat.bloque[axe] {
            corrige.vitesse[axe] = 0.0;
        }
    }
    corrige.au_sol |= resultat.au_sol;
    corrige
}

/// Corrige le dernier tick de mouvement de chaque joueur, pour qu'aucun ne traverse les
/// obstacles. S'exécute dans `FixedUpdate`, juste après la simulation du mouvement.
pub fn resoudre_les_collisions(obstacles: Query<&Obstacle>, mut joueurs: Query<&mut Physique>) {
    let obstacles: Vec<Aabb3d> = obstacles.iter().map(|obstacle| obstacle.0).collect();
    for mut physique in &mut joueurs {
        *physique = corriger(&physique, &obstacles);
    }
}

/// Le plugin du chapitre, à ajouter avec celui du chapitre 11.
pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        resoudre_les_collisions.after(simuler_le_mouvement),
    );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
