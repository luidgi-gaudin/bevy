//! Les tests du chapitre 5.
//!
//! Sans fenêtre, il n'y a pas de vrai clavier : les tests ajoutent eux-mêmes la ressource
//! `ButtonInput<KeyCode>`, et appuient sur les touches avec `press`.

use super::*;
use bevy_tutorials::outils::{app_de_test, appuyer_sur, compter, image, proche, simuler};
use core::time::Duration;

/// Un clavier sur lequel les touches données sont enfoncées.
fn clavier(touches: &[KeyCode]) -> ButtonInput<KeyCode> {
    let mut clavier = ButtonInput::default();
    for &touche in touches {
        clavier.press(touche);
    }
    clavier
}

/// Une application avec un clavier et le joueur au centre.
fn app_avec_joueur() -> App {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.world_mut().spawn((Joueur, Transform::default()));
    app
}

fn position_du_joueur(app: &mut App) -> Vec2 {
    let mut joueurs = app.world_mut().query_filtered::<&Transform, With<Joueur>>();
    joueurs.single(app.world()).unwrap().translation.truncate()
}

fn maintenir(app: &mut App, touche: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(touche);
}

// `direction` est une fonction ordinaire : on la teste sans application.

#[test]
fn sans_touche_la_direction_est_nulle() {
    assert_eq!(direction(&clavier(&[])), Vec2::ZERO);
}

#[test]
fn les_fleches_donnent_la_direction() {
    assert_eq!(direction(&clavier(&[KeyCode::ArrowUp])), Vec2::Y);
    assert_eq!(direction(&clavier(&[KeyCode::ArrowDown])), Vec2::NEG_Y);
    assert_eq!(direction(&clavier(&[KeyCode::ArrowLeft])), Vec2::NEG_X);
    assert_eq!(direction(&clavier(&[KeyCode::ArrowRight])), Vec2::X);
}

#[test]
fn les_touches_wasd_fonctionnent_aussi() {
    // `KeyCode` désigne l'emplacement de la touche : `KeyW` est la touche Z d'un clavier AZERTY.
    assert_eq!(direction(&clavier(&[KeyCode::KeyW])), Vec2::Y);
    assert_eq!(direction(&clavier(&[KeyCode::KeyS])), Vec2::NEG_Y);
    assert_eq!(direction(&clavier(&[KeyCode::KeyA])), Vec2::NEG_X);
    assert_eq!(direction(&clavier(&[KeyCode::KeyD])), Vec2::X);
}

#[test]
fn deux_touches_opposees_s_annulent() {
    let direction = direction(&clavier(&[KeyCode::ArrowLeft, KeyCode::ArrowRight]));
    assert_eq!(direction, Vec2::ZERO);
}

#[test]
fn la_diagonale_n_est_pas_plus_rapide() {
    let direction = direction(&clavier(&[KeyCode::ArrowUp, KeyCode::ArrowRight]));
    assert!(proche(direction.length(), 1.0), "{direction}");
    assert!(proche(direction.x, direction.y), "{direction}");
    assert!(direction.x > 0.0);
}

#[test]
fn le_joueur_ne_bouge_pas_sans_touche() {
    let mut app = app_avec_joueur();
    app.add_systems(Update, deplacer_le_joueur);

    simuler(&mut app, 1.0);

    assert_eq!(position_du_joueur(&mut app), Vec2::ZERO);
}

#[test]
fn le_joueur_avance_tant_qu_une_fleche_est_enfoncee() {
    let mut app = app_avec_joueur();
    app.add_systems(Update, deplacer_le_joueur);

    maintenir(&mut app, KeyCode::ArrowRight);
    simuler(&mut app, 0.5);
    let position = position_du_joueur(&mut app);
    assert!(proche(position.x, 100.0), "{position}");
    assert!(proche(position.y, 0.0), "{position}");

    // Une fois la touche relâchée, il s'arrête.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ArrowRight);
    simuler(&mut app, 0.5);
    assert!(proche(position_du_joueur(&mut app).x, 100.0));
}

#[test]
fn maj_fait_courir_deux_fois_plus_vite() {
    let mut app = app_avec_joueur();
    app.add_systems(Update, deplacer_le_joueur);

    maintenir(&mut app, KeyCode::ArrowUp);
    maintenir(&mut app, KeyCode::ShiftLeft);
    image(&mut app, Duration::from_millis(100));

    let position = position_du_joueur(&mut app);
    assert!(proche(position.y, 40.0), "{position}");
}

#[test]
fn espace_tire_un_projectile() {
    let mut app = app_avec_joueur();
    app.add_systems(Update, tirer);

    appuyer_sur(&mut app, KeyCode::Space);

    assert_eq!(compter::<Projectile>(&mut app), 1);
}

#[test]
fn garder_espace_enfonce_ne_tire_qu_un_projectile() {
    let mut app = app_avec_joueur();
    app.add_systems(Update, tirer);

    maintenir(&mut app, KeyCode::Space);
    app.update();
    // Dans un vrai jeu, le `InputPlugin` fait cela au début de chaque image : la touche reste
    // enfoncée (`pressed`), mais n'est plus « tout juste appuyée » (`just_pressed`).
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.update();
    app.update();

    assert_eq!(compter::<Projectile>(&mut app), 1);

    // Relâcher puis appuyer à nouveau tire un deuxième projectile.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::Space);
    appuyer_sur(&mut app, KeyCode::Space);
    assert_eq!(compter::<Projectile>(&mut app), 2);
}

#[test]
fn le_projectile_part_de_la_position_du_joueur() {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.world_mut()
        .spawn((Joueur, Transform::from_xyz(30.0, -20.0, 0.0)));
    app.add_systems(Update, tirer);

    appuyer_sur(&mut app, KeyCode::Space);

    let mut projectiles = app
        .world_mut()
        .query_filtered::<&Transform, With<Projectile>>();
    let projectile = projectiles.single(app.world()).unwrap();
    assert_eq!(projectile.translation, Vec3::new(30.0, -20.0, 0.0));
}

#[test]
fn le_plugin_assemble_tout() {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(plugin);

    maintenir(&mut app, KeyCode::KeyD);
    simuler(&mut app, 1.0);
    appuyer_sur(&mut app, KeyCode::Space);

    assert_eq!(compter::<Joueur>(&mut app), 1);
    assert_eq!(compter::<Projectile>(&mut app), 1);
    // 1 seconde, plus l'image de l'appui sur espace (100 ms), à 200 pixels par seconde.
    assert!(proche(position_du_joueur(&mut app).x, 220.0));
}
