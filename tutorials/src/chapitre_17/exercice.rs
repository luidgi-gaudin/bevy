//! Exercice du chapitre 17 : la compensation de latence.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_17
//! ```
//!
//! Ce chapitre utilise les solutions des chapitres 10 à 14.

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
        todo!()
    }

    /// Renvoie la position enregistrée au tick donné, ou au dernier tick enregistré avant lui.
    /// Renvoie `None` si tout l'historique est plus récent que ce tick.
    pub fn position_au_tick(&self, tick: u32) -> Option<Vec3> {
        todo!()
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
    todo!()
}

/// Passe au tick suivant, au début de chaque tick.
pub fn compter_les_ticks(mut tick: ResMut<Tick>) {
    todo!()
}

/// Enregistre la position de chaque personnage dans son [`Historique`], à chaque tick.
pub fn enregistrer_les_positions(tick: Res<Tick>) {
    todo!()
}

/// Comme le système `tirer_avec_dispersion` du chapitre 14, mais les hitbox des cibles sont
/// replacées là où le tireur les voyait, selon sa [`Latence`] (0 sans ce composant) : les cibles
/// qui ont un [`Historique`] sont placées à leur [`position_vue`].
pub fn tirer_avec_compensation(temps: Res<Time>, tick: Res<Tick>) {
    todo!()
}

/// Le plugin du chapitre. Comme celui du chapitre 14, dont il reprend la visée et le recul, il
/// remplace celui du chapitre 13 : ajoutez l'un ou l'autre, pas les deux.
///
/// Il ajoute la ressource [`Tick`], les messages, les mêmes lectures de commandes que le plugin du
/// chapitre 14, [`compter_les_ticks`] dans `FixedFirst` (au début de chaque tick), et dans
/// `FixedUpdate`, après les collisions et dans cet ordre : [`enregistrer_les_positions`],
/// `epauler_les_armes`, [`tirer_avec_compensation`] et `appliquer_le_recul`.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
