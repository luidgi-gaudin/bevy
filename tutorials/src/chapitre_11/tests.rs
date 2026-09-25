//! Les tests du chapitre 11.
//!
//! La plupart des tests appellent directement `tick_de_mouvement` : c'est une fonction ordinaire,
//! qui calcule un tick à partir de l'état précédent, facile à tester. Les derniers tests vérifient
//! le plugin complet, avec le clavier et le temps.

use super::*;
use bevy_tutorials::chapitre_10::solution::Joueur;
use bevy_tutorials::outils::{app_de_test, image, proche, simuler};
use core::f32::consts::PI;
use core::time::Duration;

/// La durée d'un tick, en secondes.
const DT: f32 = 1.0 / 128.0;

/// La durée exacte d'un tick.
const TICK: Duration = Duration::from_nanos(7_812_500);

const AVANCER: Commandes = Commandes {
    avant: 1.0,
    droite: 0.0,
    sprint: false,
    saut: false,
};

/// Un joueur immobile, au sol.
fn au_sol() -> Physique {
    Physique {
        au_sol: true,
        ..default()
    }
}

/// Simule `secondes` de mouvement tick par tick, avec les mêmes commandes.
fn simuler_ticks(
    mut physique: Physique,
    commandes: Commandes,
    lacet: f32,
    secondes: f32,
) -> Physique {
    let ticks = (secondes / DT).round() as u32;
    for _ in 0..ticks {
        physique = tick_de_mouvement(&physique, &commandes, lacet, DT);
    }
    physique
}

fn vitesse_horizontale(physique: &Physique) -> f32 {
    Vec2::new(physique.vitesse.x, physique.vitesse.z).length()
}

#[test]
fn les_axes_au_sol_suivent_le_lacet() {
    let (avant, droite) = axes_au_sol(0.0);
    assert!(avant.abs_diff_eq(Vec3::NEG_Z, 1e-6), "{avant}");
    assert!(droite.abs_diff_eq(Vec3::X, 1e-6), "{droite}");

    // Tourné de 90° vers la gauche : on regarde vers -X, et la droite est vers -Z.
    let (avant, droite) = axes_au_sol(PI / 2.0);
    assert!(avant.abs_diff_eq(Vec3::NEG_X, 1e-6), "{avant}");
    assert!(droite.abs_diff_eq(Vec3::NEG_Z, 1e-6), "{droite}");
}

#[test]
fn sans_commande_on_reste_immobile() {
    let physique = simuler_ticks(au_sol(), Commandes::default(), 0.0, 1.0);

    assert_eq!(physique.position, Vec3::ZERO);
    assert!(physique.au_sol);
}

#[test]
fn en_avancant_on_atteint_vite_la_vitesse_de_marche() {
    let physique = simuler_ticks(au_sol(), AVANCER, 0.0, 0.25);

    assert!(proche(vitesse_horizontale(&physique), VITESSE_MARCHE));
    // Vers l'avant, donc vers -Z.
    assert!(physique.vitesse.z < 0.0);
}

#[test]
fn on_avance_dans_la_direction_du_regard() {
    let physique = simuler_ticks(au_sol(), AVANCER, PI / 2.0, 1.0);

    assert!(physique.position.x < -4.0, "{}", physique.position);
    assert!(proche(physique.position.z, 0.0), "{}", physique.position);
}

#[test]
fn le_sprint_est_plus_rapide() {
    let commandes = Commandes {
        sprint: true,
        ..AVANCER
    };
    let physique = simuler_ticks(au_sol(), commandes, 0.0, 1.0);

    assert!(proche(vitesse_horizontale(&physique), VITESSE_SPRINT));
}

#[test]
fn on_ne_sprinte_pas_en_reculant() {
    let commandes = Commandes {
        avant: -1.0,
        sprint: true,
        ..default()
    };
    let physique = simuler_ticks(au_sol(), commandes, 0.0, 1.0);

    assert!(proche(vitesse_horizontale(&physique), VITESSE_MARCHE));
}

#[test]
fn la_diagonale_n_est_pas_plus_rapide() {
    let commandes = Commandes {
        droite: 1.0,
        ..AVANCER
    };
    let physique = simuler_ticks(au_sol(), commandes, 0.0, 1.0);

    assert!(proche(vitesse_horizontale(&physique), VITESSE_MARCHE));
}

#[test]
fn la_friction_arrete_le_joueur() {
    let lance = Physique {
        vitesse: Vec3::new(VITESSE_MARCHE, 0.0, 0.0),
        au_sol: true,
        ..default()
    };

    let physique = simuler_ticks(lance, Commandes::default(), 0.0, 0.5);

    assert_eq!(vitesse_horizontale(&physique), 0.0);
    // Il a glissé un peu avant de s'arrêter.
    assert!(physique.position.x > 0.5, "{}", physique.position);
}

#[test]
fn un_saut_monte_a_environ_un_metre_vingt() {
    let mut physique = au_sol();
    let sauter = Commandes {
        saut: true,
        ..default()
    };
    physique = tick_de_mouvement(&physique, &sauter, 0.0, DT);
    let mut hauteur_max = physique.position.y;
    let mut ticks_en_l_air = 1;
    while !physique.au_sol {
        physique = tick_de_mouvement(&physique, &Commandes::default(), 0.0, DT);
        hauteur_max = hauteur_max.max(physique.position.y);
        ticks_en_l_air += 1;
        assert!(ticks_en_l_air < 1000, "le joueur ne retombe jamais");
    }

    assert!((1.15..1.25).contains(&hauteur_max), "{hauteur_max} m");
    let secondes = ticks_en_l_air as f32 * DT;
    assert!((0.65..0.75).contains(&secondes), "{secondes} s en l'air");
}

#[test]
fn on_ne_peut_pas_sauter_en_l_air() {
    let en_l_air = Physique {
        position: Vec3::new(0.0, 1.0, 0.0),
        au_sol: false,
        ..default()
    };
    let sauter = Commandes {
        saut: true,
        ..default()
    };

    let physique = tick_de_mouvement(&en_l_air, &sauter, 0.0, DT);

    assert!(physique.vitesse.y < 0.0);
}

// Le plugin complet.

#[derive(Resource, Default)]
struct NombreDeTicks(u32);

/// Une application avec le plugin, un clavier et un joueur à l'origine.
fn app_avec_joueur() -> (App, Entity) {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(plugin);
    let joueur = app.world_mut().spawn(nouveau_joueur(Vec3::ZERO)).id();
    (app, joueur)
}

fn physique(app: &App, joueur: Entity) -> Physique {
    *app.world().get::<Physique>(joueur).unwrap()
}

fn maintenir(app: &mut App, touche: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(touche);
}

#[test]
fn le_jeu_simule_128_ticks_par_seconde() {
    let (mut app, _) = app_avec_joueur();
    app.init_resource::<NombreDeTicks>();
    app.add_systems(FixedUpdate, |mut ticks: ResMut<NombreDeTicks>| {
        ticks.0 += 1;
    });

    simuler(&mut app, 1.0);

    assert_eq!(app.world().resource::<NombreDeTicks>().0, 128);
}

#[test]
fn le_clavier_fait_avancer_le_joueur() {
    let (mut app, joueur) = app_avec_joueur();

    maintenir(&mut app, KeyCode::KeyW);
    simuler(&mut app, 1.0);

    let physique = physique(&app, joueur);
    assert!(physique.position.z < -4.0, "{}", physique.position);
    assert!(proche(vitesse_horizontale(&physique), VITESSE_MARCHE));
}

#[test]
fn un_appui_tres_bref_sur_espace_n_est_pas_perdu() {
    let (mut app, joueur) = app_avec_joueur();
    // Une image d'exactement un tick.
    image(&mut app, TICK);

    // Espace est enfoncé pendant une image de 2 ms, trop courte pour qu'un tick soit simulé...
    appuyer_brievement(&mut app, KeyCode::Space, Duration::from_millis(2));
    assert_eq!(physique(&app, joueur).position.y, 0.0);

    // ... mais le saut est gardé pour le tick suivant.
    image(&mut app, Duration::from_millis(100));
    assert!(physique(&app, joueur).position.y > 0.5);
}

/// Appuie sur une touche pendant une seule image de la durée donnée.
fn appuyer_brievement(app: &mut App, touche: KeyCode, duree: Duration) {
    maintenir(app, touche);
    image(app, duree);
    let mut clavier = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    clavier.release(touche);
    clavier.clear();
}

#[test]
fn le_mouvement_ne_depend_pas_des_images_par_seconde() {
    // Une seconde de jeu à 100 images par seconde...
    let (mut app_100_ips, joueur_100_ips) = app_avec_joueur();
    maintenir(&mut app_100_ips, KeyCode::KeyW);
    maintenir(&mut app_100_ips, KeyCode::KeyD);
    for _ in 0..100 {
        image(&mut app_100_ips, Duration::from_millis(10));
    }

    // ... ou à 40 images par seconde.
    let (mut app_40_ips, joueur_40_ips) = app_avec_joueur();
    maintenir(&mut app_40_ips, KeyCode::KeyW);
    maintenir(&mut app_40_ips, KeyCode::KeyD);
    for _ in 0..40 {
        image(&mut app_40_ips, Duration::from_millis(25));
    }

    // Les mêmes 128 ticks, avec les mêmes commandes : exactement la même position.
    assert_eq!(
        physique(&app_100_ips, joueur_100_ips).position,
        physique(&app_40_ips, joueur_40_ips).position
    );
}

#[test]
fn l_affichage_est_interpole_entre_deux_ticks() {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(plugin);
    let joueur = app
        .world_mut()
        .spawn((
            Joueur,
            Orientation::default(),
            Commandes::default(),
            Physique {
                position_precedente: Vec3::ZERO,
                position: Vec3::new(1.0, 0.0, 0.0),
                vitesse: Vec3::ZERO,
                au_sol: true,
            },
            Transform::default(),
        ))
        .id();

    // Une demi-durée de tick : aucun tick n'est simulé, on est à mi-chemin entre les deux.
    image(&mut app, TICK / 2);

    let transform = app.world().get::<Transform>(joueur).unwrap();
    assert!(
        proche(transform.translation.x, 0.5),
        "{}",
        transform.translation
    );
}
