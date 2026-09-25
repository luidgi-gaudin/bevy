//! Solution du chapitre 17 : la compensation de latence.

use bevy::prelude::*;
use bevy_tutorials::chapitre_10::solution::{direction_du_regard, Orientation};
use bevy_tutorials::chapitre_11::solution::{Commandes, Physique};
use bevy_tutorials::chapitre_12::solution::{resoudre_les_collisions, Obstacle};
use bevy_tutorials::chapitre_13::solution::{
    actionner_l_arme, calculer_les_degats, premier_impact, Arme, CommandesDeTir, EtatArme,
    Hitboxes, Impact, Tir, Touche, HAUTEUR_DES_YEUX,
};
use bevy_tutorials::chapitre_14::solution::{devier, dispersion, graine_de_balle, Visee};
use std::collections::VecDeque;

/// Le nombre de ticks gardés dans l'historique des positions : une seconde, à 128 ticks par
/// seconde.
pub const TICKS_D_HISTORIQUE: usize = 128;

/// Le retour dans le temps maximal, en ticks : environ 200 millisecondes à 128 ticks par seconde.
/// Au-delà, les joueurs avec une mauvaise connexion auraient trop d'avantage.
pub const RETOUR_MAX: u32 = 26;

/// Le numéro du tick en cours, depuis le début de la partie.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tick(pub u32);

/// Les positions passées d'un personnage (celles de ses pieds), pour revenir dans le temps.
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct Historique {
    /// Les positions, du tick le plus ancien au plus récent.
    positions: VecDeque<(u32, Vec3)>,
}

impl Historique {
    /// Enregistre la position du personnage au tick donné, en oubliant les positions de plus de
    /// [`TICKS_D_HISTORIQUE`] ticks.
    pub fn enregistrer(&mut self, tick: u32, position: Vec3) {
        self.positions.push_back((tick, position));
        while self.positions.len() > TICKS_D_HISTORIQUE {
            self.positions.pop_front();
        }
    }

    /// Renvoie la position enregistrée au tick donné, ou au dernier tick enregistré avant lui.
    /// Renvoie `None` si tout l'historique est plus récent que ce tick.
    pub fn position_au_tick(&self, tick: u32) -> Option<Vec3> {
        self.positions
            .iter()
            .rev()
            .find(|&&(tick_enregistre, _)| tick_enregistre <= tick)
            .map(|&(_, position)| position)
    }

    /// Le nombre de positions enregistrées.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Renvoie `true` si aucune position n'est enregistrée.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// La latence d'un joueur : le retard, en ticks, entre ce qu'il voit sur son écran et la partie
/// simulée par le serveur.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Latence(pub u32);

/// Renvoie la position d'un personnage telle que la voyait un tireur de latence `latence`, au tick
/// `tick` : celle d'il y a `latence` ticks, sans remonter de plus de [`RETOUR_MAX`] ticks. Sans
/// historique, c'est la position `actuelle`.
pub fn position_vue(historique: &Historique, actuelle: Vec3, tick: u32, latence: u32) -> Vec3 {
    let retour = latence.min(RETOUR_MAX);
    historique
        .position_au_tick(tick.saturating_sub(retour))
        .unwrap_or(actuelle)
}

/// Passe au tick suivant, au début de chaque tick.
pub fn compter_les_ticks(mut tick: ResMut<Tick>) {
    tick.0 += 1;
}

/// Enregistre la position de chaque personnage dans son [`Historique`], à chaque tick.
pub fn enregistrer_les_positions(
    tick: Res<Tick>,
    mut personnages: Query<(&Physique, &mut Historique)>,
) {
    for (physique, mut historique) in &mut personnages {
        historique.enregistrer(tick.0, physique.position);
    }
}

/// Comme le système `tirer_avec_dispersion` du chapitre 14, mais les hitbox des cibles sont
/// replacées là où le tireur les voyait, selon sa [`Latence`].
pub fn tirer_avec_compensation(
    temps: Res<Time>,
    tick: Res<Tick>,
    mut tireurs: Query<(
        Entity,
        &Physique,
        &Orientation,
        &Commandes,
        &mut CommandesDeTir,
        &Arme,
        &mut EtatArme,
        Option<&Visee>,
        Option<&Latence>,
    )>,
    personnages: Query<(Entity, &Physique, &Hitboxes, Option<&Historique>)>,
    obstacles: Query<&Obstacle>,
    mut tirs: MessageWriter<Tir>,
    mut touches: MessageWriter<Touche>,
) {
    let obstacles: Vec<Aabb3d> = obstacles.iter().map(|obstacle| obstacle.0).collect();
    for (
        tireur,
        physique,
        orientation,
        commandes,
        mut commandes_de_tir,
        arme,
        mut etat,
        visee,
        latence,
    ) in &mut tireurs
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

        // Chaque cible est replacée là où le tireur la voyait.
        let latence = latence.map_or(0, |latence| latence.0);
        let cibles = personnages
            .iter()
            .map(|(entite, physique, hitboxes, historique)| {
                let position = historique.map_or(physique.position, |historique| {
                    position_vue(historique, physique.position, tick.0, latence)
                });
                (entite, position, hitboxes)
            });
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

/// Le plugin du chapitre. Comme celui du chapitre 14, dont il reprend la visée et le recul, il
/// remplace celui du chapitre 13 : ajoutez l'un ou l'autre, pas les deux.
pub fn plugin(app: &mut App) {
    use bevy_tutorials::chapitre_11::solution::lire_le_clavier;
    use bevy_tutorials::chapitre_13::solution::lire_les_commandes_de_tir;
    use bevy_tutorials::chapitre_14::solution::{
        appliquer_le_recul, epauler_les_armes, lire_la_visee, ralentir_en_visant,
    };

    app.init_resource::<Tick>()
        .add_message::<Tir>()
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
        .add_systems(FixedFirst, compter_les_ticks)
        .add_systems(
            FixedUpdate,
            (
                enregistrer_les_positions,
                epauler_les_armes,
                tirer_avec_compensation,
                appliquer_le_recul,
            )
                .chain()
                .after(resoudre_les_collisions),
        );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
