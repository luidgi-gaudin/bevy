//! Solution du chapitre 14 : le maniement des armes, façon Call of Duty : la visée, le recul et
//! la dispersion.

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
pub fn champ_de_vision(progression: f32) -> f32 {
    CHAMP_DE_VISION
        .lerp(CHAMP_DE_VISION_EN_VISANT, progression)
        .to_radians()
}

/// Fait progresser la visée pendant `dt` secondes : vers 1 si le joueur vise, vers 0 sinon. Il
/// faut [`DUREE_POUR_EPAULER`] secondes pour passer de l'un à l'autre.
pub fn epauler(visee: &mut Visee, dt: f32) {
    let pas = dt / DUREE_POUR_EPAULER;
    visee.progression = if visee.commande {
        (visee.progression + pas).min(1.0)
    } else {
        (visee.progression - pas).max(0.0)
    };
}

/// Renvoie la dispersion d'un tir, en degrés, selon la visée et le mouvement du tireur.
///
/// La dispersion de base va de [`DISPERSION_A_LA_HANCHE`] à [`DISPERSION_EN_VISANT`] selon la
/// visée. Elle est multipliée par `1 + vitesse / VITESSE_MARCHE` (doublée en marchant), et
/// augmentée de [`DISPERSION_EN_L_AIR`] en l'air.
pub fn dispersion(progression: f32, physique: &Physique) -> f32 {
    let base = DISPERSION_A_LA_HANCHE.lerp(DISPERSION_EN_VISANT, progression);
    let vitesse = Vec2::new(physique.vitesse.x, physique.vitesse.z).length();
    let en_l_air = if physique.au_sol {
        0.0
    } else {
        DISPERSION_EN_L_AIR
    };
    base * (1.0 + vitesse / VITESSE_MARCHE) + en_l_air
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
pub fn devier(direction: Vec3, dispersion: f32, graine: u32) -> Vec3 {
    if dispersion <= 0.0 {
        return direction;
    }
    // Un point au hasard dans un disque : la racine carrée répartit les points uniformément,
    // au lieu de les concentrer au centre.
    let angle = hasard(graine).sqrt() * dispersion.to_radians();
    let tour = hasard(graine ^ 0x5BD1_E995) * TAU;
    let (axe_1, axe_2) = direction.any_orthonormal_pair();
    let (sinus_tour, cosinus_tour) = ops::sin_cos(tour);
    let cote = axe_1 * cosinus_tour + axe_2 * sinus_tour;
    let (sinus, cosinus) = ops::sin_cos(angle);
    (direction * cosinus + cote * sinus).normalize()
}

/// Lit la souris et met à jour la commande de [`Visee`] du joueur : le bouton droit vise.
pub fn lire_la_visee(
    souris: Res<ButtonInput<MouseButton>>,
    mut joueurs: Query<&mut Visee, With<Joueur>>,
) {
    for mut visee in &mut joueurs {
        visee.commande = souris.pressed(MouseButton::Right);
    }
}

/// En visant, ralentit les déplacements et empêche de sprinter : à exécuter après la lecture du
/// clavier.
pub fn ralentir_en_visant(mut joueurs: Query<(&Visee, &mut Commandes)>) {
    for (visee, mut commandes) in &mut joueurs {
        let ralentissement = 1.0_f32.lerp(RALENTISSEMENT_EN_VISANT, visee.progression);
        commandes.avant *= ralentissement;
        commandes.droite *= ralentissement;
        if visee.commande {
            commandes.sprint = false;
        }
    }
}

/// Fait progresser la visée de chaque tireur, à chaque tick.
pub fn epauler_les_armes(temps: Res<Time>, mut visees: Query<&mut Visee>) {
    for mut visee in &mut visees {
        epauler(&mut visee, temps.delta_secs());
    }
}

/// Comme le système `tirer` du chapitre 13, mais chaque balle est déviée selon la
/// [`dispersion`] : la visée et l'immobilité rendent les tirs précis.
pub fn tirer_avec_dispersion(
    temps: Res<Time>,
    mut tireurs: Query<(
        Entity,
        &Physique,
        &Orientation,
        &Commandes,
        &mut CommandesDeTir,
        &Arme,
        &mut EtatArme,
        Option<&Visee>,
    )>,
    personnages: Query<(Entity, &Physique, &Hitboxes)>,
    obstacles: Query<&Obstacle>,
    mut tirs: MessageWriter<Tir>,
    mut touches: MessageWriter<Touche>,
) {
    let obstacles: Vec<Aabb3d> = obstacles.iter().map(|obstacle| obstacle.0).collect();
    for (tireur, physique, orientation, commandes, mut commandes_de_tir, arme, mut etat, visee) in
        &mut tireurs
    {
        let sprinte = commandes.sprint && commandes.avant > 0.0;
        let tire = actionner_l_arme(
            &mut etat,
            arme,
            commandes_de_tir.tir && !sprinte,
            commandes_de_tir.recharger,
            temps.delta_secs(),
        );
        commandes_de_tir.recharger = false;
        if !tire {
            continue;
        }

        let progression = visee.map_or(0.0, |visee| visee.progression);
        let origine = physique.position + Vec3::Y * HAUTEUR_DES_YEUX;
        let direction = devier(
            direction_du_regard(orientation),
            dispersion(progression, physique),
            graine_de_balle(tireur, etat.balles_tirees),
        );
        tirs.write(Tir {
            tireur,
            origine,
            direction,
        });
        let Ok(direction) = Dir3::new(direction) else {
            continue;
        };
        let cibles = personnages
            .iter()
            .map(|(entite, physique, hitboxes)| (entite, physique.position, hitboxes));
        if let Some(Impact::Personnage {
            entite,
            zone,
            distance,
        }) = premier_impact(origine, direction, &obstacles, cibles, tireur)
        {
            touches.write(Touche {
                tireur,
                cible: entite,
                zone,
                distance,
                degats: calculer_les_degats(arme, zone, distance),
            });
        }
    }
}

/// Applique le recul de chaque balle tirée au regard du tireur. En visant, le recul est réduit de
/// moitié. Après [`DELAI_DE_RETOUR_DU_RECUL`] secondes sans tirer, le motif recommence.
pub fn appliquer_le_recul(
    temps: Res<Time>,
    mut tirs: MessageReader<Tir>,
    mut tireurs: Query<(&mut Orientation, &mut Recul, Option<&Visee>)>,
) {
    for (_, mut recul, _) in &mut tireurs {
        recul.repos += temps.delta_secs();
        if recul.repos > DELAI_DE_RETOUR_DU_RECUL {
            recul.balle = 0;
        }
    }
    for tir in tirs.read() {
        let Ok((mut orientation, mut recul, visee)) = tireurs.get_mut(tir.tireur) else {
            continue;
        };
        let Some(&dernier) = recul.motif.last() else {
            continue;
        };
        let secousse = recul.motif.get(recul.balle).copied().unwrap_or(dernier)
            * 1.0_f32.lerp(0.5, visee.map_or(0.0, |visee| visee.progression));
        orientation.tangage =
            (orientation.tangage + secousse.y.to_radians()).clamp(-TANGAGE_MAX, TANGAGE_MAX);
        orientation.lacet = normaliser_angle(orientation.lacet - secousse.x.to_radians());
        recul.balle += 1;
        recul.repos = 0.0;
    }
}

/// Le plugin du chapitre. Il remplace celui du chapitre 13 : il ne faut pas ajouter les deux.
pub fn plugin(app: &mut App) {
    app.add_message::<Tir>()
        .add_message::<Touche>()
        .add_systems(
            RunFixedMainLoop,
            (
                lire_les_commandes_de_tir,
                lire_la_visee,
                ralentir_en_visant
                    .after(lire_le_clavier)
                    .after(lire_la_visee),
            )
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(
            FixedUpdate,
            (epauler_les_armes, tirer_avec_dispersion, appliquer_le_recul)
                .chain()
                .after(resoudre_les_collisions),
        );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
