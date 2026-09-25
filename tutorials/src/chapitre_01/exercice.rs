//! Exercice du chapitre 1 : l'application, les systèmes et les ressources.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_01
//! ```
//!
//! Les types (les ressources) sont déjà écrits : les tests en ont besoin.

use bevy::prelude::*;

/// Le nombre d'images exécutées depuis le lancement du jeu.
#[derive(Resource, Default)]
pub struct CompteurImages(pub u32);

/// Le message affiché au lancement du jeu.
#[derive(Resource, Default)]
pub struct Bienvenue(pub String);

/// Les événements marquants de la partie, du plus ancien au plus récent.
#[derive(Resource, Default)]
pub struct Journal(pub Vec<String>);

/// Écrit `"Bienvenue dans Bevy !"` dans la ressource [`Bienvenue`].
///
/// C'est un système de démarrage : il ne s'exécute qu'une fois, au lancement.
pub fn ecrire_bienvenue(mut bienvenue: ResMut<Bienvenue>) {
    todo!()
}

/// Ajoute 1 au [`CompteurImages`], à chaque image.
pub fn compter_les_images(mut compteur: ResMut<CompteurImages>) {
    todo!()
}

/// Toutes les 10 images, ajoute une ligne `"<nombre> images"` au [`Journal`] : `"10 images"`,
/// `"20 images"`...
///
/// Indices : `format!("{} images", compteur.0)` crée le texte, et `compteur.0.is_multiple_of(10)`
/// dit si le compteur est un multiple de 10.
pub fn annoncer_les_paliers(compteur: Res<CompteurImages>, mut journal: ResMut<Journal>) {
    todo!()
}

/// Le plugin du chapitre : il ajoute les ressources et les systèmes à l'application.
///
/// [`annoncer_les_paliers`] doit s'exécuter après [`compter_les_images`], pour annoncer le palier
/// dès l'image où il est atteint.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
