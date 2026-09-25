//! Solution du chapitre 1 : l'application, les systèmes et les ressources.

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
    bienvenue.0 = "Bienvenue dans Bevy !".to_string();
}

/// Ajoute 1 au [`CompteurImages`], à chaque image.
pub fn compter_les_images(mut compteur: ResMut<CompteurImages>) {
    compteur.0 += 1;
}

/// Toutes les 10 images, ajoute une ligne `"<nombre> images"` au [`Journal`] : `"10 images"`,
/// `"20 images"`...
pub fn annoncer_les_paliers(compteur: Res<CompteurImages>, mut journal: ResMut<Journal>) {
    if compteur.0 > 0 && compteur.0.is_multiple_of(10) {
        journal.0.push(format!("{} images", compteur.0));
    }
}

/// Le plugin du chapitre : il ajoute les ressources et les systèmes à l'application.
///
/// [`annoncer_les_paliers`] doit s'exécuter après [`compter_les_images`], pour annoncer le palier
/// dès l'image où il est atteint.
pub fn plugin(app: &mut App) {
    app.init_resource::<CompteurImages>()
        .init_resource::<Bienvenue>()
        .init_resource::<Journal>()
        .add_systems(Startup, ecrire_bienvenue)
        .add_systems(Update, (compter_les_images, annoncer_les_paliers).chain());
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
