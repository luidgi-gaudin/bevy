//! Les tests du chapitre 7.

use super::*;
use bevy::state::app::StatesPlugin;
use bevy_tutorials::outils::{app_de_test, appuyer_sur, compter, proche, simuler};

/// Une application avec les états (`StatesPlugin`), un clavier et le plugin du chapitre.
fn app_du_jeu() -> App {
    let mut app = app_de_test();
    app.add_plugins(StatesPlugin);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(plugin);
    app
}

fn etat(app: &App) -> EtatJeu {
    *app.world().resource::<State<EtatJeu>>().get()
}

/// Passe directement à un état, sans passer par les touches du jeu.
fn aller_a(app: &mut App, etat: EtatJeu) {
    app.world_mut()
        .resource_mut::<NextState<EtatJeu>>()
        .set(etat);
    app.update();
}

#[test]
fn le_jeu_commence_dans_le_menu() {
    let mut app = app_du_jeu();
    app.update();

    assert_eq!(etat(&app), EtatJeu::Menu);
    assert_eq!(compter::<Joueur>(&mut app), 0);
}

#[test]
fn entree_lance_la_partie_a_l_image_suivante() {
    let mut app = app_du_jeu();
    app.update();

    appuyer_sur(&mut app, KeyCode::Enter);
    // Le changement d'état est demandé pendant cette image, mais n'a lieu qu'au début de la
    // suivante.
    assert_eq!(etat(&app), EtatJeu::Menu);

    app.update();
    assert_eq!(etat(&app), EtatJeu::EnJeu);
}

#[test]
fn d_autres_touches_ne_lancent_pas_la_partie() {
    let mut app = app_du_jeu();

    appuyer_sur(&mut app, KeyCode::Space);
    app.update();

    assert_eq!(etat(&app), EtatJeu::Menu);
}

#[test]
fn le_joueur_apparait_au_debut_de_la_partie() {
    let mut app = app_du_jeu();

    aller_a(&mut app, EtatJeu::EnJeu);

    assert_eq!(compter::<Joueur>(&mut app), 1);
    assert_eq!(
        app.world().resource::<TempsRestant>(),
        &TempsRestant(DUREE_PARTIE)
    );
    assert_eq!(app.world().resource::<PartiesJouees>().0, 1);
}

#[test]
fn le_temps_ne_s_ecoule_que_pendant_la_partie() {
    let mut app = app_du_jeu();
    app.insert_resource(TempsRestant(DUREE_PARTIE));

    // Dans le menu, le décompte ne tourne pas.
    simuler(&mut app, 5.0);
    assert!(proche(
        app.world().resource::<TempsRestant>().0,
        DUREE_PARTIE
    ));

    aller_a(&mut app, EtatJeu::EnJeu);
    let au_debut = app.world().resource::<TempsRestant>().0;
    simuler(&mut app, 5.0);
    let restant = app.world().resource::<TempsRestant>().0;
    assert!(proche(restant, au_debut - 5.0), "temps restant : {restant}");
}

#[test]
fn la_partie_se_termine_quand_le_temps_est_ecoule() {
    let mut app = app_du_jeu();
    aller_a(&mut app, EtatJeu::EnJeu);

    simuler(&mut app, DUREE_PARTIE - 1.0);
    assert_eq!(etat(&app), EtatJeu::EnJeu);

    simuler(&mut app, 1.5);
    assert_eq!(etat(&app), EtatJeu::FinDePartie);
    assert_eq!(app.world().resource::<TempsRestant>().0, 0.0);
}

#[test]
fn le_joueur_disparait_a_la_fin_de_la_partie() {
    let mut app = app_du_jeu();
    aller_a(&mut app, EtatJeu::EnJeu);
    assert_eq!(compter::<Joueur>(&mut app), 1);

    aller_a(&mut app, EtatJeu::FinDePartie);

    assert_eq!(compter::<Joueur>(&mut app), 0);
}

#[test]
fn on_peut_rejouer() {
    let mut app = app_du_jeu();
    aller_a(&mut app, EtatJeu::EnJeu);
    simuler(&mut app, DUREE_PARTIE + 1.0);
    assert_eq!(etat(&app), EtatJeu::FinDePartie);

    // Entrée : retour au menu, puis Entrée : nouvelle partie.
    appuyer_sur(&mut app, KeyCode::Enter);
    app.update();
    assert_eq!(etat(&app), EtatJeu::Menu);
    appuyer_sur(&mut app, KeyCode::Enter);
    app.update();
    assert_eq!(etat(&app), EtatJeu::EnJeu);

    assert_eq!(app.world().resource::<PartiesJouees>().0, 2);
    assert_eq!(compter::<Joueur>(&mut app), 1);
    // Le temps restant repart de `DUREE_PARTIE` (moins la dernière image, déjà décomptée).
    let restant = app.world().resource::<TempsRestant>().0;
    assert!(restant > DUREE_PARTIE - 1.0, "temps restant : {restant}");
}
