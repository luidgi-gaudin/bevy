//! Les tests du chapitre 10.

use super::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy_tutorials::outils::{app_de_test, image, proche};
use core::time::Duration;

/// Une application avec le plugin du chapitre et un joueur qui regarde droit devant lui.
fn app_avec_joueur(sensibilite: f32) -> (App, Entity) {
    let mut app = app_de_test();
    app.add_plugins(plugin);
    app.insert_resource(Sensibilite(sensibilite));
    let joueur = app
        .world_mut()
        .spawn((Joueur, Orientation::default(), Transform::default()))
        .id();
    (app, joueur)
}

/// Simule un mouvement de souris pendant une image de 10 ms.
fn bouger_la_souris(app: &mut App, x: f32, y: f32) {
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::new(x, y);
    image(app, Duration::from_millis(10));
}

fn orientation(app: &App, joueur: Entity) -> Orientation {
    *app.world().get::<Orientation>(joueur).unwrap()
}

#[test]
fn a_800_dpi_une_sensibilite_de_1_fait_52_cm_par_tour() {
    let distance = centimetres_par_tour(1.0, 800.0);
    assert!((distance - 51.95).abs() < 0.01, "{distance} cm");
}

#[test]
fn doubler_la_sensibilite_divise_la_distance_par_deux() {
    let distance = centimetres_par_tour(1.0, 1600.0);
    let distance_double = centimetres_par_tour(2.0, 1600.0);
    assert!(proche(distance_double * 2.0, distance));
}

#[test]
fn les_angles_sont_ramenes_entre_moins_pi_et_pi() {
    assert!(proche(normaliser_angle(0.5), 0.5));
    assert!(proche(normaliser_angle(-0.5), -0.5));
    assert!(proche(normaliser_angle(TAU + 0.5), 0.5));
    assert!(proche(normaliser_angle(PI + 0.5), 0.5 - PI));
    assert!(proche(normaliser_angle(-10.0 * TAU - 0.5), -0.5));
}

#[test]
fn par_defaut_on_regarde_vers_moins_z() {
    let direction = direction_du_regard(&Orientation::default());
    assert!(direction.abs_diff_eq(Vec3::NEG_Z, 1e-6), "{direction}");
}

#[test]
fn un_lacet_positif_tourne_vers_la_gauche() {
    let orientation = Orientation {
        lacet: PI / 2.0,
        tangage: 0.0,
    };
    let direction = direction_du_regard(&orientation);
    assert!(direction.abs_diff_eq(Vec3::NEG_X, 1e-6), "{direction}");
}

#[test]
fn un_tangage_positif_regarde_vers_le_haut() {
    let orientation = Orientation {
        lacet: 0.0,
        tangage: 0.5,
    };
    assert!(direction_du_regard(&orientation).y > 0.0);
}

#[test]
fn la_souris_vers_la_droite_tourne_le_regard_a_droite() {
    let (mut app, joueur) = app_avec_joueur(2.0);

    bouger_la_souris(&mut app, 100.0, 0.0);

    // 100 points × 0,022° × 2 = 4,4° vers la droite, donc un lacet négatif.
    let orientation = orientation(&app, joueur);
    assert!(
        proche(orientation.lacet, -4.4_f32.to_radians()),
        "{orientation:?}"
    );
    assert!(proche(orientation.tangage, 0.0));
}

#[test]
fn la_souris_vers_le_haut_fait_lever_les_yeux() {
    let (mut app, joueur) = app_avec_joueur(1.0);

    // À l'écran, l'axe vertical est orienté vers le bas : monter la souris est un déplacement
    // négatif.
    bouger_la_souris(&mut app, 0.0, -50.0);

    let orientation = orientation(&app, joueur);
    assert!(
        proche(orientation.tangage, 1.1_f32.to_radians()),
        "{orientation:?}"
    );
}

#[test]
fn on_ne_peut_pas_regarder_plus_haut_que_89_degres() {
    let (mut app, joueur) = app_avec_joueur(1.0);

    bouger_la_souris(&mut app, 0.0, -100_000.0);
    assert!(proche(orientation(&app, joueur).tangage, TANGAGE_MAX));

    bouger_la_souris(&mut app, 0.0, 100_000.0);
    assert!(proche(orientation(&app, joueur).tangage, -TANGAGE_MAX));
}

#[test]
fn le_lacet_reste_entre_moins_pi_et_pi() {
    let (mut app, joueur) = app_avec_joueur(1.0);

    // Une trentaine de tours complets vers la gauche.
    for _ in 0..100 {
        bouger_la_souris(&mut app, -5000.0, 0.0);
    }

    let lacet = orientation(&app, joueur).lacet;
    assert!((-PI..=PI).contains(&lacet), "{lacet}");
}

#[test]
fn la_visee_ne_depend_pas_du_nombre_d_images_par_seconde() {
    // Le même mouvement de souris, en 2 grandes images de 100 ms...
    let (mut app_lente, joueur_lent) = app_avec_joueur(1.0);
    for _ in 0..2 {
        app_lente
            .world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(100.0, 0.0);
        image(&mut app_lente, Duration::from_millis(100));
    }

    // ... ou en 20 petites images de 10 ms.
    let (mut app_rapide, joueur_rapide) = app_avec_joueur(1.0);
    for _ in 0..20 {
        app_rapide
            .world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(10.0, 0.0);
        image(&mut app_rapide, Duration::from_millis(10));
    }

    let lacet_lent = orientation(&app_lente, joueur_lent).lacet;
    let lacet_rapide = orientation(&app_rapide, joueur_rapide).lacet;
    assert!(
        proche(lacet_lent, lacet_rapide),
        "{lacet_lent} ≠ {lacet_rapide}"
    );
    assert!(proche(lacet_lent, -4.4_f32.to_radians()));
}

#[test]
fn seul_le_joueur_vise_avec_la_souris() {
    let (mut app, _) = app_avec_joueur(1.0);
    let ennemi = app
        .world_mut()
        .spawn((Orientation::default(), Transform::default()))
        .id();

    bouger_la_souris(&mut app, 100.0, 100.0);

    assert_eq!(orientation(&app, ennemi), Orientation::default());
}

#[test]
fn la_camera_regarde_dans_la_direction_de_l_orientation() {
    let (mut app, joueur) = app_avec_joueur(1.0);

    bouger_la_souris(&mut app, 300.0, -200.0);

    let transform = app.world().get::<Transform>(joueur).unwrap();
    let attendue = direction_du_regard(&orientation(&app, joueur));
    assert!(
        transform.forward().abs_diff_eq(attendue, 1e-5),
        "{:?} ≠ {attendue}",
        transform.forward()
    );
}
