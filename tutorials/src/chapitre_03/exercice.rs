//! Exercice du chapitre 3 : créer, modifier et détruire des entités avec les commandes.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_03
//! ```

use bevy::prelude::*;

/// Les points de vie d'une entité.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Vie(pub u32);

/// Marque les ennemis.
#[derive(Component, Default)]
pub struct Ennemi;

/// Un slime, le plus petit des ennemis.
///
/// Créer un `Slime` doit aussi ajouter [`Ennemi`], et [`Vie`] avec 3 points de vie, s'ils ne sont
/// pas déjà donnés.
///
/// Indice : c'est le rôle de l'attribut `#[require(...)]`, à ajouter sous le `#[derive]`.
#[derive(Component)]
pub struct Slime;

/// Un poison, qui retire 1 point de vie par image pendant `images_restantes` images.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Poison {
    /// Le nombre d'images pendant lesquelles le poison agit encore.
    pub images_restantes: u32,
}

/// Marque les ennemis enragés : ceux à qui il ne reste qu'un point de vie.
#[derive(Component)]
pub struct Enrage;

/// Le nombre de slimes à faire apparaître au début de la partie.
#[derive(Resource)]
pub struct TailleVague(pub u32);

/// Fait apparaître [`TailleVague`] slimes.
pub fn faire_apparaitre_la_vague(mut commands: Commands, taille: Res<TailleVague>) {
    todo!()
}

/// Retire 1 point de vie à chaque entité empoisonnée, et diminue d'une image la durée de son
/// poison. Quand le poison est épuisé, il est retiré de l'entité.
///
/// Les points de vie ne descendent jamais sous 0.
///
/// Indices :
/// - ajoutez `Entity` à une requête pour obtenir l'identifiant de chaque entité ;
/// - `commands.entity(entite).remove::<Poison>()` retire un composant ;
/// - `u32::saturating_sub` fait une soustraction qui s'arrête à 0.
pub fn appliquer_le_poison(mut commands: Commands) {
    todo!()
}

/// Ajoute le marqueur [`Enrage`] aux ennemis à qui il ne reste qu'un point de vie.
///
/// Indice : `commands.entity(entite).insert(Enrage)` ajoute un composant.
pub fn enrager_les_blesses(mut commands: Commands) {
    todo!()
}

/// Détruit les entités qui n'ont plus de points de vie.
///
/// Indice : `commands.entity(entite).despawn()` détruit une entité.
pub fn retirer_les_morts(mut commands: Commands) {
    todo!()
}

/// Le plugin du chapitre : la vague de 5 slimes au démarrage, puis à chaque image le poison, la
/// rage et la disparition des morts, dans cet ordre.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
