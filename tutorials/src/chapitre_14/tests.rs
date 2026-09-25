//! Les tests du chapitre 14.

use super::*;
use bevy_tutorials::chapitre_11::solution::{nouveau_joueur, plugin as plugin_du_mouvement};
use bevy_tutorials::chapitre_12::solution::plugin as plugin_des_collisions;
use bevy_tutorials::outils::{app_de_test, image, proche, simuler};
use core::time::Duration;

const TICK: Duration = Duration::from_nanos(7_812_500);

// La visée.

#[test]
fn epauler_prend_deux_dixiemes_de_seconde() {
    let mut visee = Visee {
        commande: true,
        progression: 0.0,
    };

    epauler(&mut visee, 0.1);
    assert!(proche(visee.progression, 0.5));
    epauler(&mut visee, 0.2);
    assert_eq!(visee.progression, 1.0);

    visee.commande = false;
    epauler(&mut visee, 0.05);
    assert!(proche(visee.progression, 0.75));
    epauler(&mut visee, 1.0);
    assert_eq!(visee.progression, 0.0);
}

#[test]
fn le_champ_de_vision_se_resserre_en_visant() {
    assert!(proche(champ_de_vision(0.0), 70.0_f32.to_radians()));
    assert!(proche(champ_de_vision(1.0), 45.0_f32.to_radians()));
    assert!(proche(champ_de_vision(0.5), 57.5_f32.to_radians()));
}

// La dispersion.

fn immobile() -> Physique {
    Physique {
        au_sol: true,
        ..default()
    }
}

#[test]
fn la_dispersion_est_plus_faible_en_visant() {
    assert!(proche(dispersion(0.0, &immobile()), DISPERSION_A_LA_HANCHE));
    assert!(proche(dispersion(1.0, &immobile()), DISPERSION_EN_VISANT));
}

#[test]
fn bouger_et_sauter_rendent_les_tirs_imprecis() {
    let en_marchant = Physique {
        vitesse: Vec3::new(VITESSE_MARCHE, 0.0, 0.0),
        au_sol: true,
        ..default()
    };
    let en_l_air = Physique {
        au_sol: false,
        ..default()
    };

    assert!(proche(
        dispersion(1.0, &en_marchant),
        2.0 * DISPERSION_EN_VISANT
    ));
    assert!(proche(
        dispersion(1.0, &en_l_air),
        DISPERSION_EN_VISANT + DISPERSION_EN_L_AIR
    ));
}

#[test]
fn le_hasard_est_entre_0_et_1_et_reproductible() {
    let nombres: Vec<f32> = (0..10_000).map(hasard).collect();

    assert!(nombres.iter().all(|nombre| (0.0..1.0).contains(nombre)));
    let moyenne = nombres.iter().sum::<f32>() / nombres.len() as f32;
    assert!((moyenne - 0.5).abs() < 0.02, "moyenne : {moyenne}");
    assert_eq!(hasard(42), hasard(42));
    assert_ne!(hasard(42), hasard(43));
}

#[test]
fn une_balle_deviee_reste_dans_le_cone_de_dispersion() {
    let regard = Vec3::new(0.3, 0.1, -1.0).normalize();
    let mut plus_grand_angle: f32 = 0.0;

    for graine in 0..2000 {
        let direction = devier(regard, 3.0, graine);
        assert!(proche(direction.length(), 1.0));
        plus_grand_angle = plus_grand_angle.max(direction.angle_between(regard).to_degrees());
    }

    assert!(plus_grand_angle <= 3.0 + 1e-3, "{plus_grand_angle}°");
    // Les balles couvrent bien tout le cône.
    assert!(plus_grand_angle > 2.8, "{plus_grand_angle}°");
}

#[test]
fn la_deviation_est_reproductible() {
    let regard = Vec3::NEG_Z;

    assert_eq!(devier(regard, 3.0, 7), devier(regard, 3.0, 7));
    assert_ne!(devier(regard, 3.0, 7), devier(regard, 3.0, 8));
    assert_eq!(devier(regard, 0.0, 7), regard);
}

#[test]
fn chaque_balle_de_chaque_tireur_a_sa_graine() {
    let tireur_1 = Entity::from_raw_u32(1).unwrap();
    let tireur_2 = Entity::from_raw_u32(2).unwrap();

    assert_ne!(graine_de_balle(tireur_1, 0), graine_de_balle(tireur_1, 1));
    assert_ne!(graine_de_balle(tireur_1, 0), graine_de_balle(tireur_2, 0));
    assert_eq!(graine_de_balle(tireur_1, 5), graine_de_balle(tireur_1, 5));
}

// Le plugin complet.

/// Une application avec les plugins des chapitres 11, 12 et 14, et un joueur armé à l'origine
/// qui regarde vers -Z.
fn app_avec_joueur() -> (App, Entity) {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.add_plugins((plugin_du_mouvement, plugin_des_collisions, plugin));
    let arme = Arme::fusil_d_assaut();
    let joueur = app
        .world_mut()
        .spawn((
            nouveau_joueur(Vec3::ZERO),
            Hitboxes::default(),
            EtatArme::charge(&arme),
            arme,
            CommandesDeTir::default(),
            Visee::default(),
            Recul::fusil_d_assaut(),
        ))
        .id();
    (app, joueur)
}

fn souris(app: &mut App) -> Mut<'_, ButtonInput<MouseButton>> {
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>()
}

fn orientation(app: &App, joueur: Entity) -> Orientation {
    *app.world().get::<Orientation>(joueur).unwrap()
}

/// Tire exactement `balles` balles, gâchette enfoncée, puis relâche la gâchette.
fn tirer_une_rafale(app: &mut App, joueur: Entity, balles: u32) {
    souris(app).press(MouseButton::Left);
    while app.world().get::<EtatArme>(joueur).unwrap().balles_tirees < balles {
        image(app, TICK);
    }
    souris(app).release(MouseButton::Left);
}

#[test]
fn le_recul_fait_monter_le_regard_selon_le_motif() {
    let (mut app, joueur) = app_avec_joueur();

    tirer_une_rafale(&mut app, joueur, 5);

    let motif = Recul::fusil_d_assaut().motif;
    let montee: f32 = motif[..5].iter().map(|secousse| secousse.y).sum();
    let cote: f32 = motif[..5].iter().map(|secousse| secousse.x).sum();
    let orientation = orientation(&app, joueur);
    assert!(
        proche(orientation.tangage, montee.to_radians()),
        "{orientation:?}"
    );
    assert!(
        proche(orientation.lacet, -cote.to_radians()),
        "{orientation:?}"
    );
}

#[test]
fn le_recul_recommence_apres_une_pause() {
    let (mut app, joueur) = app_avec_joueur();
    tirer_une_rafale(&mut app, joueur, 3);
    assert_eq!(app.world().get::<Recul>(joueur).unwrap().balle, 3);

    simuler(&mut app, 0.5);

    assert_eq!(app.world().get::<Recul>(joueur).unwrap().balle, 0);
}

#[test]
fn en_visant_le_recul_est_reduit_de_moitie() {
    let (mut app, joueur) = app_avec_joueur();
    souris(&mut app).press(MouseButton::Right);
    simuler(&mut app, 0.3);
    assert_eq!(app.world().get::<Visee>(joueur).unwrap().progression, 1.0);

    tirer_une_rafale(&mut app, joueur, 1);

    let premiere_secousse = Recul::fusil_d_assaut().motif[0];
    let tangage = orientation(&app, joueur).tangage;
    assert!(
        proche(tangage, (premiere_secousse.y / 2.0).to_radians()),
        "{tangage}"
    );
}

#[test]
fn en_visant_on_se_deplace_moins_vite_et_sans_sprinter() {
    let (mut app, joueur) = app_avec_joueur();
    souris(&mut app).press(MouseButton::Right);
    let mut clavier = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    clavier.press(KeyCode::KeyW);
    clavier.press(KeyCode::ShiftLeft);

    simuler(&mut app, 1.5);

    let vitesse = app
        .world()
        .get::<Physique>(joueur)
        .unwrap()
        .vitesse
        .length();
    assert!(
        proche(vitesse, VITESSE_MARCHE * RALENTISSEMENT_EN_VISANT),
        "{vitesse}"
    );
}

#[test]
fn la_premiere_balle_en_visant_immobile_est_presque_parfaite() {
    let (mut app, joueur) = app_avec_joueur();
    souris(&mut app).press(MouseButton::Right);
    simuler(&mut app, 0.3);

    tirer_une_rafale(&mut app, joueur, 1);

    let messages = app.world().resource::<Messages<Tir>>();
    let tir = messages
        .get_cursor()
        .read(messages)
        .next()
        .copied()
        .unwrap();
    let ecart = tir.direction.angle_between(Vec3::NEG_Z).to_degrees();
    assert!(ecart <= DISPERSION_EN_VISANT + 1e-3, "{ecart}°");
}

/// Les directions de toutes les balles tirées, enregistrées au fur et à mesure : les messages ne
/// sont gardés que deux images.
#[derive(Resource, Default)]
struct TirsEnregistres(Vec<Vec3>);

fn enregistrer_les_tirs(mut tirs: MessageReader<Tir>, mut enregistres: ResMut<TirsEnregistres>) {
    enregistres.0.extend(tirs.read().map(|tir| tir.direction));
}

#[test]
fn deux_parties_identiques_donnent_exactement_les_memes_tirs() {
    let directions = || {
        let (mut app, joueur) = app_avec_joueur();
        app.init_resource::<TirsEnregistres>();
        app.add_systems(Update, enregistrer_les_tirs);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);
        tirer_une_rafale(&mut app, joueur, 8);
        app.world_mut()
            .remove_resource::<TirsEnregistres>()
            .unwrap()
            .0
    };

    let premiere_partie = directions();
    let deuxieme_partie = directions();

    assert_eq!(premiere_partie.len(), 8);
    assert_eq!(premiere_partie, deuxieme_partie);
}
