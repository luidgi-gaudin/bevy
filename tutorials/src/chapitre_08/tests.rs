//! Les tests du chapitre 8.
//!
//! Chaque plugin se teste seul, puis le jeu complet vérifie qu'ils fonctionnent ensemble.

use super::*;
use bevy_tutorials::outils::{app_de_test, compter, image, proche};
use core::time::Duration;

/// Une application de test avec un clavier et le joueur au centre.
fn app_avec_joueur() -> App {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.world_mut().spawn((Joueur, Transform::default()));
    app
}

fn maintenir(app: &mut App, touche: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(touche);
}

fn creer_une_piece(app: &mut App, x: f32) {
    app.world_mut()
        .spawn((Piece, Transform::from_xyz(x, 0.0, 0.0)));
}

#[test]
fn les_fleches_choisissent_la_direction() {
    let mut app = app_avec_joueur();
    app.add_plugins(PluginJoueur);

    maintenir(&mut app, KeyCode::ArrowLeft);
    app.update();

    assert_eq!(app.world().resource::<Direction>(), &Direction(Vec2::NEG_X));
}

#[test]
fn le_joueur_se_deplace_dans_la_direction_choisie() {
    let mut app = app_avec_joueur();
    app.add_plugins(PluginJoueur);
    // On choisit la direction nous-mêmes, après la lecture du clavier et avant le déplacement.
    app.add_systems(
        Update,
        (|mut direction: ResMut<Direction>| direction.0 = Vec2::Y)
            .after(Etape::Entrees)
            .before(Etape::Deplacement),
    );

    image(&mut app, Duration::from_millis(100));

    let mut joueurs = app.world_mut().query_filtered::<&Transform, With<Joueur>>();
    let position = joueurs.single(app.world()).unwrap().translation;
    assert!(proche(position.y, 20.0), "{position}");
}

#[test]
fn les_pieces_rapportent_la_valeur_choisie() {
    let mut app = app_avec_joueur();
    app.add_plugins(PluginPieces { valeur: 5 });
    creer_une_piece(&mut app, 10.0);
    creer_une_piece(&mut app, -20.0);
    creer_une_piece(&mut app, 100.0);

    app.update();

    assert_eq!(app.world().resource::<Score>().0, 10);
    assert_eq!(compter::<Piece>(&mut app), 1);
}

#[test]
fn les_etapes_s_executent_dans_l_ordre() {
    let mut app = app_avec_joueur();
    app.add_plugins(PluginJeu);
    // À 40 pixels : trop loin pour être ramassée, mais le joueur s'en rapproche de 20 pixels en
    // une image de 100 ms.
    creer_une_piece(&mut app, 40.0);
    maintenir(&mut app, KeyCode::ArrowRight);

    image(&mut app, Duration::from_millis(100));

    // En une seule image : le clavier est lu, puis le joueur avance, puis il ramasse la pièce.
    assert_eq!(compter::<Piece>(&mut app), 0);
    assert_eq!(app.world().resource::<Score>().0, 1);
}

#[test]
#[should_panic(expected = "plugin was already added")]
fn un_plugin_ne_peut_etre_ajoute_qu_une_fois() {
    let mut app = app_de_test();
    app.add_plugins(PluginJeu);
    app.add_plugins(PluginJoueur);
}
