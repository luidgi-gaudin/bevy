//! Solution du chapitre 15 : le match à mort (*deathmatch*).

use bevy::prelude::*;
use bevy_tutorials::chapitre_11::solution::{Commandes, Physique};
use bevy_tutorials::chapitre_13::solution::{
    Arme, CommandesDeTir, EtatArme, Hitboxes, Touche, Zone,
};

/// Les points de vie d'un combattant en pleine forme.
pub const VIE_MAX: f32 = 100.0;

/// Le temps sans subir de dégâts, en secondes, avant que la vie ne se régénère.
pub const DELAI_DE_REGENERATION: f32 = 4.0;

/// Les points de vie régénérés par seconde.
pub const REGENERATION_PAR_SECONDE: f32 = 40.0;

/// Le temps entre une élimination et la réapparition, en secondes.
pub const DELAI_DE_REAPPARITION: f32 = 3.0;

/// Le nombre d'éliminations pour gagner le match.
pub const ELIMINATIONS_POUR_GAGNER: u32 = 20;

/// Le nombre d'éliminations gardées dans le fil des éliminations.
pub const TAILLE_DU_FIL: usize = 5;

/// Les points de vie d'un combattant.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Vie {
    /// Les points de vie restants.
    pub points: f32,
    /// Le temps écoulé depuis les derniers dégâts subis, en secondes.
    pub depuis_les_derniers_degats: f32,
}

impl Default for Vie {
    fn default() -> Self {
        Self {
            points: VIE_MAX,
            depuis_les_derniers_degats: 0.0,
        }
    }
}

/// Un combattant du match (un joueur ou un bot), et ses statistiques.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Combattant {
    /// Le nom affiché dans le fil des éliminations.
    pub nom: String,
    /// Le nombre d'adversaires éliminés.
    pub eliminations: u32,
    /// Le nombre de fois où il a été éliminé.
    pub morts: u32,
}

impl Combattant {
    /// Un nouveau combattant, sans statistiques.
    pub fn new(nom: impl Into<String>) -> Self {
        Self {
            nom: nom.into(),
            ..default()
        }
    }
}

/// Un combattant éliminé, en attente de sa réapparition.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Mort {
    /// Le temps restant avant la réapparition, en secondes.
    pub reapparition: f32,
}

/// Un point de la carte où les combattants peuvent réapparaître.
#[derive(Component, Debug, Clone, Copy)]
pub struct PointDApparition(pub Vec3);

/// Envoyé quand un combattant en élimine un autre.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Elimination {
    /// Le combattant qui a tiré la dernière balle.
    pub tueur: Entity,
    /// Le combattant éliminé.
    pub victime: Entity,
    /// La zone touchée par la dernière balle : une élimination dans la tête rapporte la gloire.
    pub zone: Zone,
}

/// Les dernières éliminations, de la plus ancienne à la plus récente, pour l'affichage.
#[derive(Resource, Debug, Default)]
pub struct FilDesEliminations(pub Vec<Elimination>);

/// L'état du match.
#[derive(Resource, Debug, Default)]
pub struct Match {
    /// Le vainqueur, une fois le match terminé.
    pub vainqueur: Option<Entity>,
}

/// Une condition : le match est en cours.
pub fn match_en_cours(le_match: Res<Match>) -> bool {
    le_match.vainqueur.is_none()
}

/// Renvoie le point d'apparition le plus éloigné des ennemis : celui dont l'ennemi le plus proche
/// est le plus loin. Sans ennemi, renvoie le premier point.
pub fn choisir_un_point_d_apparition(points: &[Vec3], ennemis: &[Vec3]) -> Option<Vec3> {
    let distance_au_plus_proche = |point: Vec3| {
        ennemis
            .iter()
            .map(|ennemi| ennemi.distance(point))
            .fold(f32::INFINITY, f32::min)
    };
    points.iter().copied().reduce(|meilleur, point| {
        if distance_au_plus_proche(point) > distance_au_plus_proche(meilleur) {
            point
        } else {
            meilleur
        }
    })
}

/// Retire les dégâts des balles aux combattants touchés, et envoie une [`Elimination`] quand l'un
/// d'eux n'a plus de points de vie. Un combattant déjà éliminé ne peut pas l'être une deuxième
/// fois.
pub fn infliger_les_degats(
    mut touches: MessageReader<Touche>,
    mut vies: Query<&mut Vie, Without<Mort>>,
    mut eliminations: MessageWriter<Elimination>,
) {
    for touche in touches.read() {
        let Ok(mut vie) = vies.get_mut(touche.cible) else {
            continue;
        };
        if vie.points <= 0.0 {
            continue;
        }
        vie.points -= touche.degats;
        vie.depuis_les_derniers_degats = 0.0;
        if vie.points <= 0.0 {
            eliminations.write(Elimination {
                tueur: touche.tireur,
                victime: touche.cible,
                zone: touche.zone,
            });
        }
    }
}

/// Tient les comptes des éliminations, met les victimes hors jeu, remplit le fil des
/// éliminations, et désigne le vainqueur.
///
/// Une victime perd ses hitbox et ses commandes jusqu'à sa réapparition : elle ne peut plus ni
/// bouger, ni tirer, ni être touchée.
pub fn enregistrer_les_eliminations(
    mut commands: Commands,
    mut eliminations: MessageReader<Elimination>,
    mut combattants: Query<&mut Combattant>,
    mut fil: ResMut<FilDesEliminations>,
    mut le_match: ResMut<Match>,
) {
    for elimination in eliminations.read() {
        if let Ok(mut victime) = combattants.get_mut(elimination.victime) {
            victime.morts += 1;
        }
        if elimination.tueur != elimination.victime
            && let Ok(mut tueur) = combattants.get_mut(elimination.tueur)
        {
            tueur.eliminations += 1;
            if tueur.eliminations >= ELIMINATIONS_POUR_GAGNER && le_match.vainqueur.is_none() {
                le_match.vainqueur = Some(elimination.tueur);
            }
        }
        commands
            .entity(elimination.victime)
            .insert(Mort {
                reapparition: DELAI_DE_REAPPARITION,
            })
            .remove::<(Hitboxes, Commandes, CommandesDeTir)>();
        fil.0.push(*elimination);
        if fil.0.len() > TAILLE_DU_FIL {
            fil.0.remove(0);
        }
    }
}

/// Régénère la vie des combattants qui n'ont pas subi de dégâts depuis
/// [`DELAI_DE_REGENERATION`] secondes, jusqu'à [`VIE_MAX`].
pub fn regenerer(temps: Res<Time>, mut vies: Query<&mut Vie, Without<Mort>>) {
    let dt = temps.delta_secs();
    for mut vie in &mut vies {
        vie.depuis_les_derniers_degats += dt;
        if vie.depuis_les_derniers_degats >= DELAI_DE_REGENERATION {
            vie.points = (vie.points + REGENERATION_PAR_SECONDE * dt).min(VIE_MAX);
        }
    }
}

/// Fait réapparaître les combattants éliminés, après [`DELAI_DE_REAPPARITION`] secondes, au point
/// d'apparition le plus éloigné des autres combattants : en pleine forme, chargeur plein, avec
/// leurs hitbox et leurs commandes.
pub fn faire_reapparaitre(
    mut commands: Commands,
    temps: Res<Time>,
    mut morts: Query<(
        Entity,
        &mut Mort,
        &mut Physique,
        &mut Vie,
        Option<(&Arme, &mut EtatArme)>,
    )>,
    vivants: Query<&Physique, Without<Mort>>,
    points: Query<&PointDApparition>,
) {
    let points: Vec<Vec3> = points.iter().map(|point| point.0).collect();
    let ennemis: Vec<Vec3> = vivants.iter().map(|physique| physique.position).collect();
    for (entite, mut mort, mut physique, mut vie, arme) in &mut morts {
        mort.reapparition -= temps.delta_secs();
        if mort.reapparition > 0.0 {
            continue;
        }
        let position = choisir_un_point_d_apparition(&points, &ennemis).unwrap_or(Vec3::ZERO);
        *physique = Physique {
            position,
            position_precedente: position,
            vitesse: Vec3::ZERO,
            au_sol: true,
        };
        *vie = Vie::default();
        if let Some((arme, mut etat)) = arme {
            *etat = EtatArme {
                balles_tirees: etat.balles_tirees,
                ..EtatArme::charge(arme)
            };
        }
        commands.entity(entite).remove::<Mort>().insert((
            Hitboxes::default(),
            Commandes::default(),
            CommandesDeTir::default(),
        ));
    }
}

/// Le plugin du chapitre. Ses systèmes s'exécutent dans `FixedPostUpdate`, après chaque tick de
/// `FixedUpdate` : ils fonctionnent avec le tir du chapitre 13 comme avec celui du chapitre 14.
pub fn plugin(app: &mut App) {
    app.add_message::<Elimination>()
        .init_resource::<FilDesEliminations>()
        .init_resource::<Match>()
        .add_systems(
            FixedPostUpdate,
            (
                infliger_les_degats,
                enregistrer_les_eliminations,
                regenerer,
                faire_reapparaitre,
            )
                .chain()
                .run_if(match_en_cours),
        );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
