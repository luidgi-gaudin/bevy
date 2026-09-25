//! Les tests du chapitre 15.

use super::*;
use bevy_tutorials::chapitre_11::solution::{
    nouveau_joueur, plugin as plugin_du_mouvement, TICKS_PAR_SECONDE,
};
use bevy_tutorials::chapitre_12::solution::plugin as plugin_des_collisions;
use bevy_tutorials::chapitre_14::solution::{plugin as plugin_du_maniement, Recul, Visee};
use bevy_tutorials::outils::{app_de_test, image, proche, simuler};
use core::time::Duration;

const TICK: Duration = Duration::from_nanos(7_812_500);

// Les points d'apparition.

#[test]
fn on_reapparait_au_point_le_plus_eloigne_des_ennemis() {
    let points = [
        Vec3::ZERO,
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(20.0, 0.0, 0.0),
    ];
    // L'ennemi le plus proche est à 1 m du premier point, à 8 m du deuxième, à 2 m du troisième.
    let ennemis = [Vec3::new(1.0, 0.0, 0.0), Vec3::new(18.0, 0.0, 0.0)];

    assert_eq!(
        choisir_un_point_d_apparition(&points, &ennemis),
        Some(Vec3::new(10.0, 0.0, 0.0))
    );
}

#[test]
fn sans_ennemi_le_premier_point_convient() {
    let points = [Vec3::new(5.0, 0.0, 0.0), Vec3::ZERO];

    assert_eq!(
        choisir_un_point_d_apparition(&points, &[]),
        Some(Vec3::new(5.0, 0.0, 0.0))
    );
    assert_eq!(choisir_un_point_d_apparition(&[], &[Vec3::ZERO]), None);
}

// Les systèmes du match.

/// Une application avec le plugin du chapitre, à 128 ticks par seconde.
fn app_du_match() -> App {
    let mut app = app_de_test();
    app.insert_resource(Time::<Fixed>::from_hz(TICKS_PAR_SECONDE));
    app.add_message::<Touche>();
    app.add_plugins(plugin);
    app
}

/// Un combattant en pleine forme, armé, à la position donnée.
fn combattant(app: &mut App, nom: &str, position: Vec3) -> Entity {
    let arme = Arme::fusil_d_assaut();
    app.world_mut()
        .spawn((
            Combattant::new(nom),
            Vie::default(),
            Hitboxes::default(),
            Commandes::default(),
            CommandesDeTir::default(),
            EtatArme::charge(&arme),
            arme,
            Physique {
                position,
                position_precedente: position,
                vitesse: Vec3::ZERO,
                au_sol: true,
            },
        ))
        .id()
}

fn toucher(app: &mut App, tireur: Entity, cible: Entity, degats: f32) {
    app.world_mut().write_message(Touche {
        tireur,
        cible,
        zone: Zone::Torse,
        distance: 10.0,
        degats,
    });
}

fn eliminer(app: &mut App, tueur: Entity, victime: Entity) {
    toucher(app, tueur, victime, 1000.0);
    image(app, TICK);
}

fn combattant_de(app: &App, entite: Entity) -> Combattant {
    app.world().get::<Combattant>(entite).unwrap().clone()
}

#[test]
fn les_degats_retirent_des_points_de_vie() {
    let mut app = app_du_match();
    let tireur = combattant(&mut app, "Tireur", Vec3::ZERO);
    let cible = combattant(&mut app, "Cible", Vec3::X);

    toucher(&mut app, tireur, cible, 30.0);
    image(&mut app, TICK);

    assert_eq!(app.world().get::<Vie>(cible).unwrap().points, 70.0);
}

#[test]
fn a_zero_point_de_vie_le_combattant_est_elimine() {
    let mut app = app_du_match();
    let tireur = combattant(&mut app, "Tireur", Vec3::ZERO);
    let cible = combattant(&mut app, "Cible", Vec3::X);

    for _ in 0..4 {
        toucher(&mut app, tireur, cible, 30.0);
    }
    image(&mut app, TICK);

    // La victime est hors jeu : ni hitbox, ni commandes.
    assert!(app.world().get::<Mort>(cible).is_some());
    assert!(app.world().get::<Hitboxes>(cible).is_none());
    assert!(app.world().get::<Commandes>(cible).is_none());
    assert!(app.world().get::<CommandesDeTir>(cible).is_none());
    // Les statistiques et le fil des éliminations.
    assert_eq!(combattant_de(&app, tireur).eliminations, 1);
    assert_eq!(combattant_de(&app, cible).morts, 1);
    let fil = &app.world().resource::<FilDesEliminations>().0;
    assert_eq!(fil.len(), 1);
    assert_eq!(fil[0].tueur, tireur);
    assert_eq!(fil[0].victime, cible);
}

#[test]
fn un_combattant_n_est_elimine_qu_une_fois() {
    let mut app = app_du_match();
    let tireur = combattant(&mut app, "Tireur", Vec3::ZERO);
    let cible = combattant(&mut app, "Cible", Vec3::X);

    for _ in 0..10 {
        toucher(&mut app, tireur, cible, 30.0);
    }
    image(&mut app, TICK);
    toucher(&mut app, tireur, cible, 30.0);
    image(&mut app, TICK);

    assert_eq!(combattant_de(&app, tireur).eliminations, 1);
    assert_eq!(combattant_de(&app, cible).morts, 1);
}

#[test]
fn la_vie_se_regenere_apres_quatre_secondes_sans_degats() {
    let mut app = app_du_match();
    let tireur = combattant(&mut app, "Tireur", Vec3::ZERO);
    let cible = combattant(&mut app, "Cible", Vec3::X);
    toucher(&mut app, tireur, cible, 50.0);
    image(&mut app, TICK);

    simuler(&mut app, 3.8);
    assert_eq!(app.world().get::<Vie>(cible).unwrap().points, 50.0);

    simuler(&mut app, 0.5);
    let points = app.world().get::<Vie>(cible).unwrap().points;
    assert!(points > 50.0 && points < VIE_MAX, "{points}");

    simuler(&mut app, 2.0);
    assert_eq!(app.world().get::<Vie>(cible).unwrap().points, VIE_MAX);
}

#[test]
fn les_degats_repoussent_la_regeneration() {
    let mut app = app_du_match();
    let tireur = combattant(&mut app, "Tireur", Vec3::ZERO);
    let cible = combattant(&mut app, "Cible", Vec3::X);
    toucher(&mut app, tireur, cible, 50.0);
    image(&mut app, TICK);
    simuler(&mut app, 3.0);

    toucher(&mut app, tireur, cible, 10.0);
    image(&mut app, TICK);
    simuler(&mut app, 3.0);

    assert_eq!(app.world().get::<Vie>(cible).unwrap().points, 40.0);
}

#[test]
fn on_reapparait_apres_trois_secondes_loin_des_ennemis() {
    let mut app = app_du_match();
    let tireur = combattant(&mut app, "Tireur", Vec3::ZERO);
    let cible = combattant(&mut app, "Cible", Vec3::X);
    app.world_mut()
        .spawn(PointDApparition(Vec3::new(2.0, 0.0, 0.0)));
    app.world_mut()
        .spawn(PointDApparition(Vec3::new(30.0, 0.0, 0.0)));
    app.world_mut()
        .get_mut::<EtatArme>(cible)
        .unwrap()
        .munitions = 3;

    eliminer(&mut app, tireur, cible);
    simuler(&mut app, 2.8);
    assert!(app.world().get::<Mort>(cible).is_some());

    simuler(&mut app, 0.3);
    assert!(app.world().get::<Mort>(cible).is_none());
    assert!(app.world().get::<Hitboxes>(cible).is_some());
    assert!(app.world().get::<Commandes>(cible).is_some());
    assert_eq!(app.world().get::<Vie>(cible).unwrap().points, VIE_MAX);
    assert_eq!(app.world().get::<EtatArme>(cible).unwrap().munitions, 30);
    assert_eq!(
        app.world().get::<Physique>(cible).unwrap().position,
        Vec3::new(30.0, 0.0, 0.0)
    );
}

#[test]
fn le_fil_ne_garde_que_les_dernieres_eliminations() {
    let mut app = app_du_match();
    let tireur = combattant(&mut app, "Tireur", Vec3::ZERO);
    let victimes: Vec<Entity> = (0..7)
        .map(|numero| combattant(&mut app, &format!("Victime {numero}"), Vec3::X))
        .collect();

    for &victime in &victimes {
        eliminer(&mut app, tireur, victime);
    }

    let fil = &app.world().resource::<FilDesEliminations>().0;
    assert_eq!(fil.len(), TAILLE_DU_FIL);
    assert_eq!(fil.last().unwrap().victime, victimes[6]);
    assert_eq!(fil[0].victime, victimes[2]);
}

#[test]
fn le_premier_a_vingt_eliminations_gagne_le_match() {
    let mut app = app_du_match();
    let champion = combattant(&mut app, "Champion", Vec3::ZERO);
    let adversaire = combattant(&mut app, "Adversaire", Vec3::X);
    app.world_mut()
        .get_mut::<Combattant>(champion)
        .unwrap()
        .eliminations = ELIMINATIONS_POUR_GAGNER - 1;

    eliminer(&mut app, champion, adversaire);

    assert_eq!(app.world().resource::<Match>().vainqueur, Some(champion));

    // Le match est terminé : plus rien ne se passe.
    let autre = combattant(&mut app, "Autre", Vec3::Y);
    eliminer(&mut app, champion, autre);
    assert_eq!(combattant_de(&app, autre).morts, 0);
}

#[test]
fn un_duel_de_bout_en_bout() {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.add_plugins((
        plugin_du_mouvement,
        plugin_des_collisions,
        plugin_du_maniement,
        plugin,
    ));
    let arme = Arme::fusil_d_assaut();
    let joueur = app
        .world_mut()
        .spawn((
            nouveau_joueur(Vec3::ZERO),
            Combattant::new("Joueur"),
            Vie::default(),
            Hitboxes::default(),
            EtatArme::charge(&arme),
            arme,
            CommandesDeTir::default(),
            Visee::default(),
            Recul::fusil_d_assaut(),
        ))
        .id();
    let cible = combattant(&mut app, "Cible", Vec3::new(0.0, 0.0, -5.0));
    app.world_mut()
        .spawn(PointDApparition(Vec3::new(0.0, 0.0, 40.0)));

    // Le joueur épaule, puis tire dans la tête de la cible, à 5 mètres.
    let mut souris = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    souris.press(MouseButton::Right);
    simuler(&mut app, 0.3);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    simuler(&mut app, 0.5);

    // Deux balles dans la tête suffisent.
    assert_eq!(combattant_de(&app, joueur).eliminations, 1);
    let fil = &app.world().resource::<FilDesEliminations>().0;
    assert_eq!(fil[0].zone, Zone::Tete);

    // La cible réapparaît, loin du joueur.
    simuler(&mut app, 3.0);
    assert!(app.world().get::<Mort>(cible).is_none());
    let position = app.world().get::<Physique>(cible).unwrap().position;
    assert!(proche(position.z, 40.0), "{position}");
}
