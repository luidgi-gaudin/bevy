//! Les tests du chapitre 4.
//!
//! Pour que les tests donnent toujours le même résultat, le temps n'avance pas tout seul : les
//! outils `image` et `simuler` exécutent des images d'une durée choisie.

use super::*;
use bevy_tutorials::outils::{app_de_test, compter, image, proche, simuler};
use core::time::Duration;

/// Crée une entité immobile au centre, avec une vitesse de 100 pixels par seconde vers la droite.
fn creer_un_mobile(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((Transform::default(), Vitesse(Vec2::new(100.0, 0.0))))
        .id()
}

fn abscisse(app: &App, entite: Entity) -> f32 {
    app.world().get::<Transform>(entite).unwrap().translation.x
}

#[test]
fn les_entites_avancent_proportionnellement_au_temps() {
    let mut app = app_de_test();
    app.add_systems(Update, deplacer);
    let mobile = creer_un_mobile(&mut app);

    simuler(&mut app, 0.5);

    let x = abscisse(&app, mobile);
    assert!(proche(x, 50.0), "après 0,5 s à 100 pixels/s : {x}");
}

#[test]
fn la_vitesse_ne_depend_pas_du_nombre_d_images() {
    // Une seconde en 10 images de 100 ms...
    let mut app_rapide = app_de_test();
    app_rapide.add_systems(Update, deplacer);
    let mobile_rapide = creer_un_mobile(&mut app_rapide);
    for _ in 0..10 {
        image(&mut app_rapide, Duration::from_millis(100));
    }

    // ... ou en 4 images de 250 ms.
    let mut app_lente = app_de_test();
    app_lente.add_systems(Update, deplacer);
    let mobile_lent = creer_un_mobile(&mut app_lente);
    for _ in 0..4 {
        image(&mut app_lente, Duration::from_millis(250));
    }

    assert!(proche(abscisse(&app_rapide, mobile_rapide), 100.0));
    assert!(proche(abscisse(&app_lente, mobile_lent), 100.0));
}

#[test]
fn une_piece_apparait_toutes_les_deux_secondes() {
    let mut app = app_de_test();
    app.init_resource::<MinuteurPieces>();
    app.add_systems(Update, faire_apparaitre_des_pieces);

    simuler(&mut app, 1.9);
    assert_eq!(compter::<Piece>(&mut app), 0);

    simuler(&mut app, 0.1);
    assert_eq!(compter::<Piece>(&mut app), 1);

    simuler(&mut app, 4.0);
    assert_eq!(compter::<Piece>(&mut app), 3);
}

#[test]
fn les_pieces_ont_une_duree_de_vie() {
    let mut app = app_de_test();
    app.init_resource::<MinuteurPieces>();
    app.add_systems(Update, faire_apparaitre_des_pieces);

    simuler(&mut app, 2.0);

    let mut durees = app.world_mut().query_filtered::<&DureeDeVie, With<Piece>>();
    let duree = durees.single(app.world()).unwrap();
    assert_eq!(duree.0.duration(), Duration::from_secs(5));
    assert_eq!(duree.0.mode(), TimerMode::Once);
}

#[test]
fn les_entites_disparaissent_a_la_fin_de_leur_duree_de_vie() {
    let mut app = app_de_test();
    app.add_systems(Update, faire_disparaitre);
    app.world_mut()
        .spawn((Piece, DureeDeVie(Timer::from_seconds(5.0, TimerMode::Once))));

    simuler(&mut app, 4.9);
    assert_eq!(compter::<Piece>(&mut app), 1);

    simuler(&mut app, 0.1);
    assert_eq!(compter::<Piece>(&mut app), 0);
}

#[test]
fn le_chrono_mesure_le_temps_ecoule() {
    let mut app = app_de_test();
    app.init_resource::<Chrono>();
    app.add_systems(Update, mettre_a_jour_le_chrono);

    simuler(&mut app, 3.0);

    let chrono = app.world().resource::<Chrono>().0;
    assert!(proche(chrono, 3.0), "chrono : {chrono}");
}

#[test]
fn le_plugin_assemble_tout() {
    let mut app = app_de_test();
    app.add_plugins(plugin);
    let mobile = creer_un_mobile(&mut app);

    simuler(&mut app, 10.0);

    // Les pièces apparues à 2 s et à 4 s ont déjà disparu, pas celles de 6, 8 et 10 s.
    assert_eq!(compter::<Piece>(&mut app), 3);
    assert!(proche(abscisse(&app, mobile), 1000.0));
    assert!(proche(app.world().resource::<Chrono>().0, 10.0));
}
