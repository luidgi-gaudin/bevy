//! Les tests du chapitre 12.

use super::*;
use bevy_tutorials::chapitre_11::solution::{
    nouveau_joueur, plugin as plugin_du_mouvement, tick_de_mouvement, Commandes, VITESSE_MARCHE,
};
use bevy_tutorials::outils::{app_de_test, proche, simuler};

const DT: f32 = 1.0 / 128.0;

/// Un mur d'un mètre d'épaisseur, face à la direction +X, qui commence à `x = 1`.
fn mur_en_x() -> Aabb3d {
    Obstacle::new(Vec3::new(1.5, 1.5, 0.0), Vec3::new(1.0, 3.0, 20.0)).0
}

/// Une caisse d'un mètre de haut, de `z = -3,8` à `z = -1,8`.
fn caisse() -> Aabb3d {
    Obstacle::new(Vec3::new(0.0, 0.5, -2.8), Vec3::new(4.0, 1.0, 2.0)).0
}

#[test]
fn la_boite_du_joueur_part_de_ses_pieds() {
    let boite = boite_du_joueur(Vec3::new(1.0, 2.0, 3.0));

    assert!(Vec3::from(boite.min).abs_diff_eq(Vec3::new(0.6, 2.0, 2.6), 1e-5));
    assert!(Vec3::from(boite.max).abs_diff_eq(Vec3::new(1.4, 3.8, 3.4), 1e-5));
}

#[test]
fn deux_boites_qui_se_touchent_ne_se_chevauchent_pas() {
    let a = Aabb3d::new(Vec3::ZERO, Vec3::ONE);
    let collee = Aabb3d::new(Vec3::new(2.0, 0.0, 0.0), Vec3::ONE);
    let chevauchante = Aabb3d::new(Vec3::new(1.9, 0.0, 0.0), Vec3::ONE);

    assert!(!se_chevauchent(&a, &collee));
    assert!(se_chevauchent(&a, &chevauchante));
}

#[test]
fn sans_obstacle_le_deplacement_est_libre() {
    let resultat = deplacer_avec_collisions(Vec3::ZERO, Vec3::new(1.0, 0.5, -2.0), &[]);

    assert_eq!(resultat.position, Vec3::new(1.0, 0.5, -2.0));
    assert_eq!(resultat.bloque, [false; 3]);
}

#[test]
fn on_ne_traverse_pas_un_mur() {
    let resultat = deplacer_avec_collisions(Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0), &[mur_en_x()]);

    // Arrêté au contact : la face avant de sa boîte touche le mur.
    assert!(proche(resultat.position.x, 1.0 - DEMI_LARGEUR_JOUEUR));
    assert_eq!(resultat.bloque, [true, false, false]);
}

#[test]
fn on_glisse_le_long_d_un_mur() {
    let resultat = deplacer_avec_collisions(Vec3::ZERO, Vec3::new(2.0, 0.0, 1.5), &[mur_en_x()]);

    assert!(proche(resultat.position.x, 1.0 - DEMI_LARGEUR_JOUEUR));
    assert!(proche(resultat.position.z, 1.5));
}

#[test]
fn toucher_un_mur_n_empeche_pas_de_le_longer() {
    let contre_le_mur = Vec3::new(1.0 - DEMI_LARGEUR_JOUEUR, 0.0, 0.0);

    let resultat =
        deplacer_avec_collisions(contre_le_mur, Vec3::new(0.0, 0.0, -3.0), &[mur_en_x()]);

    assert!(proche(resultat.position.z, -3.0));
    assert_eq!(resultat.bloque, [false; 3]);
}

#[test]
fn on_se_pose_sur_une_caisse() {
    let au_dessus = Vec3::new(0.0, 1.5, -2.8);

    let resultat = deplacer_avec_collisions(au_dessus, Vec3::new(0.0, -1.0, 0.0), &[caisse()]);

    assert_eq!(resultat.position.y, 1.0);
    assert!(resultat.au_sol);
    assert_eq!(resultat.bloque, [false, true, false]);
}

#[test]
fn on_se_cogne_la_tete_au_plafond() {
    let plafond = Obstacle::new(Vec3::new(0.0, 3.5, 0.0), Vec3::new(10.0, 1.0, 10.0)).0;

    let resultat = deplacer_avec_collisions(Vec3::ZERO, Vec3::new(0.0, 2.0, 0.0), &[plafond]);

    // La tête (à 1,8 m au-dessus des pieds) touche le plafond, qui commence à 3 m.
    assert!(proche(resultat.position.y, 3.0 - HAUTEUR_JOUEUR));
    assert!(!resultat.au_sol);
}

#[test]
fn corriger_annule_la_vitesse_contre_un_mur() {
    let physique = Physique {
        position_precedente: Vec3::ZERO,
        position: Vec3::new(2.0, 0.0, 0.5),
        vitesse: Vec3::new(5.0, 0.0, 1.0),
        au_sol: true,
    };

    let corrige = corriger(&physique, &[mur_en_x()]);

    assert_eq!(corrige.vitesse, Vec3::new(0.0, 0.0, 1.0));
    assert!(proche(corrige.position.x, 1.0 - DEMI_LARGEUR_JOUEUR));
}

/// Simule `secondes` de mouvement avec collisions, tick par tick : le saut n'est demandé qu'au
/// premier tick.
fn simuler_avec_collisions(mut physique: Physique, sauter: bool, secondes: f32) -> Physique {
    let obstacles = [caisse()];
    for tick in 0..(secondes / DT).round() as u32 {
        let commandes = Commandes {
            avant: 1.0,
            saut: sauter && tick == 0,
            ..default()
        };
        physique = tick_de_mouvement(&physique, &commandes, 0.0, DT);
        physique = corriger(&physique, &obstacles);
    }
    physique
}

/// Un joueur lancé vers la caisse, à la vitesse de marche.
fn joueur_lance() -> Physique {
    Physique {
        vitesse: Vec3::new(0.0, 0.0, -VITESSE_MARCHE),
        au_sol: true,
        ..default()
    }
}

#[test]
fn sans_sauter_la_caisse_bloque_le_joueur() {
    let physique = simuler_avec_collisions(joueur_lance(), false, 1.0);

    assert!(proche(physique.position.z, -1.8 + DEMI_LARGEUR_JOUEUR));
    assert_eq!(physique.position.y, 0.0);
}

#[test]
fn en_sautant_on_monte_sur_la_caisse() {
    let physique = simuler_avec_collisions(joueur_lance(), true, 0.6);

    assert_eq!(physique.position.y, 1.0);
    assert!(physique.au_sol);
    assert!(physique.position.z < -1.8);
}

#[test]
fn le_plugin_arrete_le_joueur_contre_un_mur() {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins((plugin_du_mouvement, plugin));
    // Un mur devant le joueur, qui regarde vers -Z.
    app.world_mut().spawn(Obstacle::new(
        Vec3::new(0.0, 1.5, -3.5),
        Vec3::new(10.0, 3.0, 1.0),
    ));
    let joueur = app.world_mut().spawn(nouveau_joueur(Vec3::ZERO)).id();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    simuler(&mut app, 2.0);

    let physique = app.world().get::<Physique>(joueur).unwrap();
    assert!(proche(physique.position.z, -3.0 + DEMI_LARGEUR_JOUEUR));
    assert_eq!(physique.vitesse.z, 0.0);
}
