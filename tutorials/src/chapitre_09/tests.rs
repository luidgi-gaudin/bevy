//! Les tests du projet final.
//!
//! Ce sont surtout des tests d'intégration : ils jouent des parties entières, en appuyant sur les
//! touches et en faisant avancer le temps, et vérifient ce qui se passe.

use super::*;
use bevy_tutorials::outils::{
    app_de_test, appuyer_sur, compter, image, proche, simuler, DUREE_IMAGE,
};

fn app_du_jeu() -> App {
    let mut app = app_de_test();
    app.add_plugins(JeuPlugin);
    app
}

fn etat(app: &App) -> EtatJeu {
    *app.world().resource::<State<EtatJeu>>().get()
}

fn score(app: &App) -> u32 {
    app.world().resource::<Score>().0
}

/// Appuie sur Entrée, puis exécute l'image où la partie commence.
fn commencer_une_partie(app: &mut App) {
    appuyer_sur(app, KeyCode::Enter);
    image(app, DUREE_IMAGE);
    assert_eq!(etat(app), EtatJeu::EnJeu);
}

fn position_du_joueur(app: &mut App) -> Vec2 {
    let mut joueurs = app.world_mut().query_filtered::<&Transform, With<Joueur>>();
    joueurs.single(app.world()).unwrap().translation.truncate()
}

/// Retire le joueur, pour qu'il ne ramasse pas les pièces par hasard.
fn retirer_le_joueur(app: &mut App) {
    let mut joueurs = app.world_mut().query_filtered::<Entity, With<Joueur>>();
    let joueur = joueurs.single(app.world()).unwrap();
    app.world_mut().despawn(joueur);
}

fn creer_une_piece(app: &mut App, position: Vec2) {
    app.world_mut().spawn((
        Piece,
        Transform::from_translation(position.extend(0.0)),
        DespawnOnExit(EtatJeu::EnJeu),
    ));
}

fn maintenir(app: &mut App, touche: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(touche);
}

// Le hasard

#[test]
fn le_hasard_donne_des_nombres_entre_0_et_1() {
    let mut hasard = Hasard::default();
    let nombres: Vec<f32> = (0..10_000).map(|_| hasard.nombre()).collect();

    assert!(nombres.iter().all(|nombre| (0.0..1.0).contains(nombre)));
    let moyenne = nombres.iter().sum::<f32>() / nombres.len() as f32;
    assert!((moyenne - 0.5).abs() < 0.02, "moyenne : {moyenne}");
}

#[test]
fn le_hasard_est_reproductible() {
    let suite = |graine| {
        let mut hasard = Hasard(graine);
        (0..10).map(|_| hasard.nombre()).collect::<Vec<f32>>()
    };

    assert_eq!(suite(42), suite(42));
    assert_ne!(suite(42), suite(43));
}

#[test]
fn les_positions_sont_dans_l_arene() {
    let mut hasard = Hasard::default();
    for _ in 0..1000 {
        let position = hasard.position_dans_l_arene();
        assert!(
            position.x.abs() <= DEMI_LARGEUR_ARENE - TAILLE_PIECE,
            "{position}"
        );
        assert!(
            position.y.abs() <= DEMI_HAUTEUR_ARENE - TAILLE_PIECE,
            "{position}"
        );
    }
}

// Le déroulement d'une partie

#[test]
fn le_jeu_attend_dans_le_menu() {
    let mut app = app_du_jeu();

    simuler(&mut app, 5.0);

    assert_eq!(etat(&app), EtatJeu::Menu);
    assert_eq!(compter::<Joueur>(&mut app), 0);
    assert_eq!(compter::<Piece>(&mut app), 0);
}

#[test]
fn entree_lance_une_partie() {
    let mut app = app_du_jeu();

    commencer_une_partie(&mut app);

    assert_eq!(compter::<Joueur>(&mut app), 1);
    assert_eq!(position_du_joueur(&mut app), Vec2::ZERO);
    assert_eq!(score(&app), 0);
}

#[test]
fn le_joueur_se_deplace_avec_le_clavier() {
    let mut app = app_du_jeu();
    commencer_une_partie(&mut app);

    maintenir(&mut app, KeyCode::ArrowRight);
    simuler(&mut app, 0.5);

    let position = position_du_joueur(&mut app);
    assert!(proche(position.x, 150.0), "{position}");
    assert!(proche(position.y, 0.0), "{position}");
}

#[test]
fn le_joueur_ne_sort_pas_de_l_arene() {
    let mut app = app_du_jeu();
    commencer_une_partie(&mut app);

    maintenir(&mut app, KeyCode::ArrowLeft);
    maintenir(&mut app, KeyCode::ArrowUp);
    simuler(&mut app, 5.0);

    let position = position_du_joueur(&mut app);
    assert!(proche(
        position.x,
        -(DEMI_LARGEUR_ARENE - TAILLE_JOUEUR / 2.0)
    ));
    assert!(proche(position.y, DEMI_HAUTEUR_ARENE - TAILLE_JOUEUR / 2.0));
}

#[test]
fn une_piece_apparait_chaque_seconde_jusqu_au_maximum() {
    let mut app = app_du_jeu();
    commencer_une_partie(&mut app);
    retirer_le_joueur(&mut app);

    simuler(&mut app, 3.0);
    assert_eq!(compter::<Piece>(&mut app), 3);

    simuler(&mut app, 10.0);
    assert_eq!(compter::<Piece>(&mut app), PIECES_MAX);
}

#[test]
fn les_pieces_apparaissent_dans_l_arene() {
    let mut app = app_du_jeu();
    commencer_une_partie(&mut app);
    retirer_le_joueur(&mut app);

    simuler(&mut app, 5.0);

    let mut pieces = app.world_mut().query_filtered::<&Transform, With<Piece>>();
    for piece in pieces.iter(app.world()) {
        assert!(piece.translation.x.abs() < DEMI_LARGEUR_ARENE);
        assert!(piece.translation.y.abs() < DEMI_HAUTEUR_ARENE);
    }
}

#[test]
fn ramasser_une_piece_rapporte_un_point() {
    let mut app = app_du_jeu();
    commencer_une_partie(&mut app);
    creer_une_piece(&mut app, Vec2::new(20.0, 0.0));
    creer_une_piece(&mut app, Vec2::new(0.0, -25.0));

    image(&mut app, DUREE_IMAGE);

    assert_eq!(score(&app), 2);
}

#[test]
fn la_partie_dure_trente_secondes() {
    let mut app = app_du_jeu();
    commencer_une_partie(&mut app);

    simuler(&mut app, DUREE_PARTIE - 1.0);
    assert_eq!(etat(&app), EtatJeu::EnJeu);

    simuler(&mut app, 2.0);
    assert_eq!(etat(&app), EtatJeu::FinDePartie);
    // Le joueur et les pièces disparaissent avec la partie.
    assert_eq!(compter::<Joueur>(&mut app), 0);
    assert_eq!(compter::<Piece>(&mut app), 0);
}

#[test]
fn le_meilleur_score_est_conserve() {
    let mut app = app_du_jeu();

    // Une première partie, avec au moins 3 pièces ramassées.
    commencer_une_partie(&mut app);
    for x in [10.0, 20.0, -20.0] {
        creer_une_piece(&mut app, Vec2::new(x, 0.0));
    }
    simuler(&mut app, DUREE_PARTIE + 1.0);
    assert_eq!(etat(&app), EtatJeu::FinDePartie);
    let premier_score = score(&app);
    assert!(premier_score >= 3);
    assert_eq!(app.world().resource::<MeilleurScore>().0, premier_score);

    // Une deuxième partie, sans pièce placée : le score repart de zéro.
    commencer_une_partie(&mut app);
    assert_eq!(score(&app), 0);
    retirer_le_joueur(&mut app);
    simuler(&mut app, DUREE_PARTIE + 1.0);
    assert_eq!(etat(&app), EtatJeu::FinDePartie);
    assert_eq!(score(&app), 0);

    // Le meilleur score est celui de la première partie.
    assert_eq!(app.world().resource::<MeilleurScore>().0, premier_score);
}
