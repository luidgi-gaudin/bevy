//! Les tests du chapitre 16.

use super::*;
use bevy_tutorials::chapitre_11::solution::{plugin as plugin_du_mouvement, TICKS_PAR_SECONDE};
use bevy_tutorials::chapitre_12::solution::plugin as plugin_des_collisions;
use bevy_tutorials::chapitre_13::solution::{plugin as plugin_du_tir, Arme, EtatArme, Tir};
use bevy_tutorials::chapitre_15::solution::{plugin as plugin_du_match, Vie};
use bevy_tutorials::outils::{app_de_test, proche, simuler};
use core::f32::consts::PI;

// L'orientation.

#[test]
fn regarder_vers_un_point() {
    let droit_devant = orientation_vers(Vec3::ZERO, Vec3::new(0.0, 0.0, -10.0));
    assert!(proche(droit_devant.lacet, 0.0) && proche(droit_devant.tangage, 0.0));

    let a_gauche = orientation_vers(Vec3::ZERO, Vec3::new(-10.0, 0.0, 0.0));
    assert!(proche(a_gauche.lacet, PI / 2.0), "{a_gauche:?}");

    let en_haut = orientation_vers(Vec3::ZERO, Vec3::new(0.0, 10.0, -10.0));
    assert!(proche(en_haut.tangage, PI / 4.0), "{en_haut:?}");
}

#[test]
fn tourner_vers_limite_la_vitesse_de_rotation() {
    let depart = Orientation::default();
    let voulue = Orientation {
        lacet: PI / 2.0,
        tangage: -0.05,
    };

    let orientation = tourner_vers(depart, voulue, 0.1);

    assert!(proche(orientation.lacet, 0.1));
    assert!(proche(orientation.tangage, -0.05));
}

#[test]
fn tourner_vers_prend_le_chemin_le_plus_court() {
    // De 170° à -170°, le plus court est de passer par 180° : 20°, et pas 340° dans l'autre sens.
    let depart = Orientation {
        lacet: 170.0_f32.to_radians(),
        tangage: 0.0,
    };
    let voulue = Orientation {
        lacet: (-170.0_f32).to_radians(),
        tangage: 0.0,
    };

    let orientation = tourner_vers(depart, voulue, 1.0);

    assert!(proche(orientation.lacet, voulue.lacet), "{orientation:?}");
}

#[test]
fn un_mur_bloque_la_ligne_de_vue() {
    let mur = Obstacle::new(Vec3::new(0.0, 1.5, -5.0), Vec3::new(4.0, 3.0, 0.5)).0;
    let yeux = Vec3::new(0.0, 1.6, 0.0);

    assert!(!ligne_de_vue(yeux, Vec3::new(0.0, 1.6, -10.0), &[mur]));
    assert!(ligne_de_vue(yeux, Vec3::new(0.0, 1.6, -4.0), &[mur]));
    assert!(ligne_de_vue(yeux, Vec3::new(10.0, 1.6, -10.0), &[mur]));
}

// Les bots en action.

/// Une application avec les plugins des chapitres 11, 12, 13, 15 et 16, à 128 ticks par seconde.
fn app_des_bots() -> App {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(Time::<Fixed>::from_hz(TICKS_PAR_SECONDE));
    app.add_plugins((
        plugin_du_mouvement,
        plugin_des_collisions,
        plugin_du_tir,
        plugin_du_match,
        plugin,
    ));
    app
}

/// Un bot armé, à la position donnée, qui regarde vers -Z.
fn bot(app: &mut App, niveau: Niveau, position: Vec3) -> Entity {
    let arme = Arme::fusil_d_assaut();
    app.world_mut()
        .spawn((
            Bot::new(niveau),
            Combattant::new("Bot"),
            Vie::default(),
            Hitboxes::default(),
            Orientation::default(),
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
            Transform::from_translation(position),
        ))
        .id()
}

/// Un ennemi immobile, sans arme.
fn ennemi(app: &mut App, position: Vec3) -> Entity {
    app.world_mut()
        .spawn((
            Combattant::new("Ennemi"),
            Vie::default(),
            Hitboxes::default(),
            Physique {
                position,
                position_precedente: position,
                vitesse: Vec3::ZERO,
                au_sol: true,
            },
        ))
        .id()
}

/// Les balles tirées, enregistrées au fur et à mesure.
#[derive(Resource, Default)]
struct Tirs(u32);

fn compter_les_tirs(mut tirs: MessageReader<Tir>, mut compte: ResMut<Tirs>) {
    compte.0 += tirs.read().count() as u32;
}

fn tirs(app: &App) -> u32 {
    app.world().resource::<Tirs>().0
}

#[test]
fn un_bot_se_tourne_vers_un_ennemi_visible_et_le_suit() {
    let mut app = app_des_bots();
    let le_bot = bot(&mut app, Niveau::FACILE, Vec3::ZERO);
    // Sans arme : le bot vise, mais ne peut pas éliminer l'ennemi.
    app.world_mut()
        .entity_mut(le_bot)
        .remove::<(Arme, EtatArme)>();
    let cible = ennemi(&mut app, Vec3::new(10.0, 0.0, 0.0));

    simuler(&mut app, 1.5);

    // Le bot s'est décalé sur le côté en combattant, mais son regard suit toujours la tête de la
    // cible.
    let yeux = app.world().get::<Physique>(le_bot).unwrap().position + Vec3::Y * HAUTEUR_DES_YEUX;
    let point_vise = app.world().get::<Physique>(cible).unwrap().position
        + Vec3::Y * Niveau::FACILE.hauteur_visee;
    let regard = direction_du_regard(app.world().get::<Orientation>(le_bot).unwrap());
    let ecart = regard.angle_between(point_vise - yeux).to_degrees();
    assert!(ecart < 1.0, "{ecart}°");
    assert!(yeux.z.abs() > 0.5, "le bot ne s'est pas décalé : {yeux}");
}

#[test]
fn un_bot_ne_voit_pas_a_travers_les_murs() {
    let mut app = app_des_bots();
    app.init_resource::<Tirs>();
    app.add_systems(Update, compter_les_tirs);
    let le_bot = bot(&mut app, Niveau::DIFFICILE, Vec3::ZERO);
    ennemi(&mut app, Vec3::new(0.0, 0.0, -10.0));
    app.world_mut().spawn(Obstacle::new(
        Vec3::new(0.0, 1.5, -5.0),
        Vec3::new(6.0, 3.0, 0.5),
    ));

    simuler(&mut app, 2.0);

    assert_eq!(app.world().get::<Bot>(le_bot).unwrap().cible, None);
    assert_eq!(tirs(&app), 0);
}

#[test]
fn un_bot_attend_son_temps_de_reaction_avant_de_tirer() {
    let mut app = app_des_bots();
    app.init_resource::<Tirs>();
    app.add_systems(Update, compter_les_tirs);
    bot(&mut app, Niveau::FACILE, Vec3::ZERO);
    // Droit devant, à hauteur de torse : le bot n'a même pas besoin de tourner.
    ennemi(&mut app, Vec3::new(0.0, 0.4, -10.0));

    simuler(&mut app, Niveau::FACILE.temps_de_reaction - 0.1);
    assert_eq!(tirs(&app), 0);

    simuler(&mut app, 0.3);
    assert!(tirs(&app) > 0);
}

#[test]
fn un_bot_difficile_elimine_une_cible_immobile() {
    let mut app = app_des_bots();
    let le_bot = bot(&mut app, Niveau::DIFFICILE, Vec3::ZERO);
    let cible = ennemi(&mut app, Vec3::new(8.0, 0.0, -8.0));

    simuler(&mut app, 2.0);

    assert!(app.world().get::<Mort>(cible).is_some());
    assert_eq!(
        app.world().get::<Combattant>(le_bot).unwrap().eliminations,
        1
    );
}

#[test]
fn sans_ennemi_un_bot_patrouille() {
    let mut app = app_des_bots();
    let le_bot = bot(&mut app, Niveau::FACILE, Vec3::ZERO);
    app.world_mut()
        .spawn(PointDApparition(Vec3::new(0.0, 0.0, -10.0)));
    app.world_mut()
        .spawn(PointDApparition(Vec3::new(10.0, 0.0, -10.0)));

    // Assez longtemps pour atteindre le premier point de passage.
    simuler(&mut app, 3.0);
    assert_eq!(app.world().get::<Bot>(le_bot).unwrap().point_de_passage, 1);

    // Puis il repart vers le suivant.
    simuler(&mut app, 2.0);
    let position = app.world().get::<Physique>(le_bot).unwrap().position;
    assert!(position.x > 2.0, "{position}");
}
