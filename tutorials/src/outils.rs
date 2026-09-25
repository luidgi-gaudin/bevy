//! Des outils pour écrire les tests des tutoriels.
//!
//! Ils sont utilisés par les tests de tous les chapitres, et vous pouvez vous en servir dans vos
//! propres tests. Le guide `TESTER.md` explique comment ils fonctionnent.

use bevy::{platform::time::Instant, prelude::*, time::TimeUpdateStrategy};
use core::time::Duration;

/// La durée d'une image simulée par [`simuler`] : 100 millisecondes.
pub const DUREE_IMAGE: Duration = Duration::from_millis(100);

/// Crée une application de test, sans fenêtre ni rendu.
///
/// Elle contient les [`MinimalPlugins`], qui gèrent en particulier le temps ([`Time`]). Le temps
/// n'avance que quand on le demande, avec [`image`] ou [`simuler`], pour que les tests donnent
/// toujours le même résultat.
pub fn app_de_test() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

/// Exécute une seule image de l'application, qui dure `duree`.
///
/// Bevy limite la durée d'une image à 250 millisecondes : au-delà, le temps du jeu n'avance que de
/// 250 millisecondes.
pub fn image(app: &mut App, duree: Duration) {
    let mut temps_reel = app.world_mut().resource_mut::<Time<Real>>();
    if temps_reel.first_update().is_none() {
        // Sans cela, la toute première image aurait une durée nulle.
        temps_reel.update_with_instant(Instant::now());
    }
    app.insert_resource(TimeUpdateStrategy::ManualDuration(duree));
    app.update();
}

/// Exécute des images de 100 millisecondes, jusqu'à avoir simulé `secondes` secondes.
pub fn simuler(app: &mut App, secondes: f32) {
    let images = (secondes / DUREE_IMAGE.as_secs_f32()).round() as u32;
    for _ in 0..images {
        image(app, DUREE_IMAGE);
    }
}

/// Simule un appui bref sur une touche : elle est enfoncée pendant une image de 100 millisecondes,
/// puis relâchée.
///
/// L'application doit contenir la ressource `ButtonInput<KeyCode>` : ajoutez-la avec
/// `app.init_resource::<ButtonInput<KeyCode>>()`.
pub fn appuyer_sur(app: &mut App, touche: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(touche);
    image(app, DUREE_IMAGE);
    let mut clavier = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    clavier.release(touche);
    // Sans le `InputPlugin`, personne ne remet à zéro les touches « tout juste appuyées » à chaque
    // image : on le fait nous-mêmes.
    clavier.clear();
}

/// Renvoie `true` si `a` et `b` sont égaux, à un millième près.
///
/// Les calculs sur les nombres à virgule (`f32`) font des erreurs d'arrondi : il ne faut pas
/// tester s'ils sont exactement égaux.
pub fn proche(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

/// Compte les entités qui ont le composant `C`.
pub fn compter<C: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), With<C>>()
        .iter(app.world())
        .count()
}
