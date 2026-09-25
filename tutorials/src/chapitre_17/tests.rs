//! Les tests du chapitre 17.

use super::*;
use bevy_tutorials::chapitre_11::solution::{nouveau_joueur, plugin as plugin_du_mouvement};
use bevy_tutorials::chapitre_12::solution::plugin as plugin_des_collisions;
use bevy_tutorials::outils::{app_de_test, image, simuler};
use core::time::Duration;

const TICK: Duration = Duration::from_nanos(7_812_500);

// L'historique.

#[test]
fn l_historique_garde_les_positions_passees() {
    let mut historique = Historique::default();
    for tick in 1..=10 {
        historique.enregistrer(tick, Vec3::new(tick as f32, 0.0, 0.0));
    }

    assert_eq!(
        historique.position_au_tick(4),
        Some(Vec3::new(4.0, 0.0, 0.0))
    );
    assert_eq!(
        historique.position_au_tick(10),
        Some(Vec3::new(10.0, 0.0, 0.0))
    );
    // Après le dernier tick enregistré : la dernière position connue.
    assert_eq!(
        historique.position_au_tick(15),
        Some(Vec3::new(10.0, 0.0, 0.0))
    );
    // Avant le premier : aucune.
    assert_eq!(historique.position_au_tick(0), None);
}

#[test]
fn l_historique_oublie_les_positions_trop_anciennes() {
    let mut historique = Historique::default();
    for tick in 1..=200 {
        historique.enregistrer(tick, Vec3::new(tick as f32, 0.0, 0.0));
    }

    assert_eq!(historique.len(), TICKS_D_HISTORIQUE);
    assert_eq!(historique.position_au_tick(50), None);
    assert_eq!(
        historique.position_au_tick(73),
        Some(Vec3::new(73.0, 0.0, 0.0))
    );
}

#[test]
fn on_voit_les_autres_avec_le_retard_de_sa_latence() {
    let mut historique = Historique::default();
    for tick in 1..=100 {
        historique.enregistrer(tick, Vec3::new(tick as f32, 0.0, 0.0));
    }
    let actuelle = Vec3::new(100.0, 0.0, 0.0);

    assert_eq!(position_vue(&historique, actuelle, 100, 0), actuelle);
    assert_eq!(
        position_vue(&historique, actuelle, 100, 10),
        Vec3::new(90.0, 0.0, 0.0)
    );
}

#[test]
fn on_ne_revient_pas_plus_de_200_ms_en_arriere() {
    let mut historique = Historique::default();
    for tick in 1..=100 {
        historique.enregistrer(tick, Vec3::new(tick as f32, 0.0, 0.0));
    }

    let vue = position_vue(&historique, Vec3::ZERO, 100, 60);

    assert_eq!(vue, Vec3::new((100 - RETOUR_MAX) as f32, 0.0, 0.0));
}

#[test]
fn sans_historique_on_voit_la_position_actuelle() {
    let actuelle = Vec3::new(5.0, 0.0, 0.0);

    assert_eq!(
        position_vue(&Historique::default(), actuelle, 100, 10),
        actuelle
    );
}

// Le tir compensé.

/// Une application avec les plugins des chapitres 11, 12 et 17, un tireur à l'origine qui épaule
/// en regardant vers -Z avec la latence donnée, et une cible à 10 m devant lui.
fn app_avec_tireur_et_cible(latence: u32) -> (App, Entity, Entity) {
    let mut app = app_de_test();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.add_plugins((plugin_du_mouvement, plugin_des_collisions, plugin));
    let arme = Arme::fusil_d_assaut();
    let tireur = app
        .world_mut()
        .spawn((
            nouveau_joueur(Vec3::ZERO),
            Hitboxes::default(),
            EtatArme::charge(&arme),
            arme,
            CommandesDeTir::default(),
            Visee::default(),
            Latence(latence),
        ))
        .id();
    let cible = app
        .world_mut()
        .spawn((
            Physique {
                position: Vec3::new(0.0, 0.0, -10.0),
                position_precedente: Vec3::new(0.0, 0.0, -10.0),
                vitesse: Vec3::ZERO,
                au_sol: true,
            },
            Hitboxes::default(),
            Historique::default(),
        ))
        .id();
    // Le tireur épaule, pour que ses tirs soient précis.
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    simuler(&mut app, 0.5);
    (app, tireur, cible)
}

/// Téléporte la cible de 3 m sur le côté : hors de la ligne de mire du tireur.
fn deplacer_la_cible(app: &mut App, cible: Entity) {
    let mut physique = app.world_mut().get_mut::<Physique>(cible).unwrap();
    physique.position = Vec3::new(3.0, 0.0, -10.0);
    physique.position_precedente = physique.position;
}

/// Tire une seule balle, et renvoie les cibles touchées.
fn tirer_une_balle(app: &mut App) -> Vec<Entity> {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    image(app, TICK);
    let messages = app.world().resource::<Messages<Touche>>();
    messages
        .get_cursor()
        .read(messages)
        .map(|touche| touche.cible)
        .collect()
}

#[test]
fn le_tir_touche_la_cible_la_ou_le_tireur_la_voyait() {
    let (mut app, _, cible) = app_avec_tireur_et_cible(10);

    // La cible s'écarte, mais le tireur, avec 10 ticks de retard, la voit encore devant lui.
    deplacer_la_cible(&mut app, cible);
    image(&mut app, TICK);

    assert_eq!(tirer_une_balle(&mut app), [cible]);
}

#[test]
fn sans_latence_la_meme_balle_rate() {
    let (mut app, _, cible) = app_avec_tireur_et_cible(0);

    deplacer_la_cible(&mut app, cible);
    image(&mut app, TICK);

    assert!(tirer_une_balle(&mut app).is_empty());
}

#[test]
fn au_dela_de_200_ms_la_compensation_s_arrete() {
    // 50 ticks de latence : près de 400 ms.
    let (mut app, _, cible) = app_avec_tireur_et_cible(50);

    // La cible s'est écartée il y a 40 ticks : plus que le retour maximal de 26 ticks.
    deplacer_la_cible(&mut app, cible);
    for _ in 0..40 {
        image(&mut app, TICK);
    }

    assert!(tirer_une_balle(&mut app).is_empty());
}

#[test]
fn les_ticks_sont_comptes() {
    let (mut app, _, cible) = app_avec_tireur_et_cible(0);
    let avant = app.world().resource::<Tick>().0;

    simuler(&mut app, 1.0);

    assert_eq!(app.world().resource::<Tick>().0, avant + 128);
    assert_eq!(
        app.world().get::<Historique>(cible).unwrap().len(),
        TICKS_D_HISTORIQUE
    );
}
