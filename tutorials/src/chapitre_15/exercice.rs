//! Exercice du chapitre 15 : le match à mort (*deathmatch*).
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_15
//! ```
//!
//! Ce chapitre utilise les solutions des chapitres 11 et 13.

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
    todo!()
}

/// Renvoie le point d'apparition le plus éloigné des ennemis : celui dont l'ennemi le plus proche
/// est le plus loin. Sans ennemi, renvoie le premier point.
pub fn choisir_un_point_d_apparition(points: &[Vec3], ennemis: &[Vec3]) -> Option<Vec3> {
    todo!()
}

/// Retire les dégâts des balles aux combattants touchés, et envoie une [`Elimination`] quand l'un
/// d'eux n'a plus de points de vie. Un combattant déjà éliminé ne peut pas l'être une deuxième
/// fois, même si plusieurs balles le touchent pendant le même tick.
///
/// Les dégâts remettent à zéro le temps écoulé depuis les derniers dégâts (pour la régénération).
pub fn infliger_les_degats(mut touches: MessageReader<Touche>) {
    todo!()
}

/// Tient les comptes des éliminations, met les victimes hors jeu, remplit le fil des
/// éliminations, et désigne le vainqueur.
///
/// Une victime perd ses hitbox et ses commandes jusqu'à sa réapparition : elle ne peut plus ni
/// bouger, ni tirer, ni être touchée.
///
/// Indices : `commands.entity(e).insert(...).remove::<(A, B, C)>()` ajoute et retire des
/// composants ; un combattant qui s'élimine lui-même ne gagne pas d'élimination.
pub fn enregistrer_les_eliminations(mut commands: Commands) {
    todo!()
}

/// Régénère la vie des combattants qui n'ont pas subi de dégâts depuis
/// [`DELAI_DE_REGENERATION`] secondes, jusqu'à [`VIE_MAX`].
pub fn regenerer(temps: Res<Time>) {
    todo!()
}

/// Fait réapparaître les combattants éliminés, après [`DELAI_DE_REAPPARITION`] secondes, au point
/// d'apparition le plus éloigné des autres combattants : en pleine forme, chargeur plein, avec
/// leurs hitbox et leurs commandes.
pub fn faire_reapparaitre(mut commands: Commands, temps: Res<Time>) {
    todo!()
}

/// Le plugin du chapitre : le message, les deux ressources, et les quatre systèmes enchaînés dans
/// `FixedPostUpdate` (après chaque tick de `FixedUpdate`), seulement tant que le match est en
/// cours.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
