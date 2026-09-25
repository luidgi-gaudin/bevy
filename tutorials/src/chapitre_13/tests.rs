//! Les tests du chapitre 13.

use super::*;
use bevy_tutorials::chapitre_11::solution::{nouveau_joueur, plugin as plugin_du_mouvement};
use bevy_tutorials::chapitre_12::solution::plugin as plugin_des_collisions;
use bevy_tutorials::outils::{app_de_test, image, proche};
use core::time::Duration;

const DT: f32 = 1.0 / 128.0;
const TICK: Duration = Duration::from_nanos(7_812_500);

/// Les yeux d'un joueur dont les pieds sont à l'origine.
const YEUX: Vec3 = Vec3::new(0.0, HAUTEUR_DES_YEUX, 0.0);

// Les dégâts.

#[test]
fn les_degats_dependent_de_la_zone_touchee() {
    let arme = Arme::fusil_d_assaut();

    assert!(proche(calculer_les_degats(&arme, Zone::Tete, 10.0), 60.0));
    assert!(proche(calculer_les_degats(&arme, Zone::Torse, 10.0), 30.0));
    assert!(proche(calculer_les_degats(&arme, Zone::Jambes, 10.0), 24.0));
}

#[test]
fn les_degats_diminuent_au_dela_de_la_portee_efficace() {
    let arme = Arme::fusil_d_assaut();

    assert!(proche(calculer_les_degats(&arme, Zone::Torse, 25.0), 30.0));
    // À mi-chemin entre la portée efficace et son double : 80 % des dégâts.
    assert!(proche(calculer_les_degats(&arme, Zone::Torse, 37.5), 24.0));
    assert!(proche(calculer_les_degats(&arme, Zone::Torse, 50.0), 18.0));
    assert!(proche(calculer_les_degats(&arme, Zone::Torse, 200.0), 18.0));
}

// Les impacts.

/// Une liste de personnages, avec les hitbox par défaut, pour `premier_impact`.
fn personnages(pieds: &[(Entity, Vec3)], hitboxes: &Hitboxes) -> Vec<(Entity, Vec3, Hitboxes)> {
    pieds
        .iter()
        .map(|&(entite, position)| (entite, position, hitboxes.clone()))
        .collect()
}

fn impact(
    origine: Vec3,
    direction: Vec3,
    obstacles: &[Aabb3d],
    cibles: &[(Entity, Vec3)],
) -> Option<Impact> {
    let tireur = Entity::from_raw_u32(1000).unwrap();
    let personnages = personnages(cibles, &Hitboxes::default());
    premier_impact(
        origine,
        Dir3::new(direction).unwrap(),
        obstacles,
        personnages
            .iter()
            .map(|(entite, pieds, hitboxes)| (*entite, *pieds, hitboxes)),
        tireur,
    )
}

fn entite(numero: u32) -> Entity {
    Entity::from_raw_u32(numero).unwrap()
}

#[test]
fn viser_a_hauteur_des_yeux_touche_la_tete() {
    let cible = entite(1);

    let impact = impact(
        YEUX,
        Vec3::NEG_Z,
        &[],
        &[(cible, Vec3::new(0.0, 0.0, -10.0))],
    );

    assert_eq!(
        impact,
        Some(Impact::Personnage {
            entite: cible,
            zone: Zone::Tete,
            // La face avant de la tête est 12 cm devant le centre du personnage.
            distance: 10.0 - 0.12,
        })
    );
}

#[test]
fn viser_plus_bas_touche_le_torse_puis_les_jambes() {
    let cible = entite(1);
    let pieds = Vec3::new(0.0, 0.0, -10.0);

    let torse = impact(
        Vec3::new(0.0, 1.2, 0.0),
        Vec3::NEG_Z,
        &[],
        &[(cible, pieds)],
    );
    let jambes = impact(
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::NEG_Z,
        &[],
        &[(cible, pieds)],
    );

    assert!(matches!(
        torse,
        Some(Impact::Personnage {
            zone: Zone::Torse,
            ..
        })
    ));
    assert!(matches!(
        jambes,
        Some(Impact::Personnage {
            zone: Zone::Jambes,
            ..
        })
    ));
}

#[test]
fn un_tir_a_cote_ne_touche_rien() {
    let impact = impact(
        YEUX,
        Vec3::new(1.0, 0.0, -1.0),
        &[],
        &[(entite(1), Vec3::new(0.0, 0.0, -10.0))],
    );

    assert_eq!(impact, None);
}

#[test]
fn un_mur_arrete_la_balle() {
    let mur = Obstacle::new(Vec3::new(0.0, 1.5, -5.0), Vec3::new(4.0, 3.0, 0.5)).0;

    let impact = impact(
        YEUX,
        Vec3::NEG_Z,
        &[mur],
        &[(entite(1), Vec3::new(0.0, 0.0, -10.0))],
    );

    assert_eq!(impact, Some(Impact::Obstacle { distance: 4.75 }));
}

#[test]
fn la_balle_touche_le_personnage_le_plus_proche() {
    let loin = entite(1);
    let proche_du_tireur = entite(2);

    let impact = impact(
        YEUX,
        Vec3::NEG_Z,
        &[],
        &[
            (loin, Vec3::new(0.0, 0.0, -20.0)),
            (proche_du_tireur, Vec3::new(0.0, 0.0, -8.0)),
        ],
    );

    assert!(matches!(
        impact,
        Some(Impact::Personnage { entite, .. }) if entite == proche_du_tireur
    ));
}

#[test]
fn on_ne_se_touche_pas_soi_meme() {
    let tireur = entite(1000);
    let hitboxes = Hitboxes::default();

    // Le tireur, à l'origine, tire depuis ses propres yeux, qui sont dans sa hitbox de tête.
    let impact = premier_impact(
        YEUX,
        Dir3::NEG_Z,
        &[],
        [(tireur, Vec3::ZERO, &hitboxes)],
        tireur,
    );

    assert_eq!(impact, None);
}

// La gâchette.

/// Actionne l'arme pendant `secondes`, gâchette enfoncée, et compte les balles tirées.
fn tirer_pendant(etat: &mut EtatArme, arme: &Arme, secondes: f32) -> u32 {
    let mut balles = 0;
    for _ in 0..(secondes / DT).round() as u32 {
        if actionner_l_arme(etat, arme, true, false, DT) {
            balles += 1;
        }
    }
    balles
}

#[test]
fn la_cadence_de_tir_est_respectee_exactement() {
    // Un chargeur immense, pour ne pas recharger.
    let arme = Arme {
        taille_chargeur: 1000,
        ..Arme::fusil_d_assaut()
    };
    let mut etat = EtatArme::charge(&arme);

    // 600 tirs par minute : 10 par seconde. Une balle tout de suite, puis une toutes les 100 ms.
    let balles = tirer_pendant(&mut etat, &arme, 5.95);

    assert_eq!(balles, 60);
}

#[test]
fn tirer_par_petites_rafales_n_est_pas_plus_rapide() {
    let arme = Arme::fusil_d_assaut();
    let mut etat = EtatArme::charge(&arme);

    // La gâchette enfoncée un tick sur deux, pendant 0,95 seconde.
    let mut balles = 0;
    for tick in 0..(0.95 / DT).round() as u32 {
        if actionner_l_arme(&mut etat, &arme, tick % 2 == 0, false, DT) {
            balles += 1;
        }
    }

    assert!(balles <= 10, "{balles} balles");
}

#[test]
fn un_chargeur_vide_se_recharge_automatiquement() {
    let arme = Arme::fusil_d_assaut();
    let mut etat = EtatArme::charge(&arme);

    // 30 balles en 2,9 secondes...
    assert_eq!(tirer_pendant(&mut etat, &arme, 2.95), 30);
    assert_eq!(etat.munitions, 0);

    // ... puis 2 secondes de rechargement, sans tirer...
    assert_eq!(tirer_pendant(&mut etat, &arme, 1.9), 0);
    assert!(etat.rechargement.is_some());

    // ... et on tire à nouveau dès la fin du rechargement...
    let mut ticks = 0;
    while !actionner_l_arme(&mut etat, &arme, true, false, DT) {
        ticks += 1;
        assert!(ticks < 128, "le rechargement ne se termine jamais");
    }
    assert_eq!(etat.munitions, 29);
    // ... à la cadence normale : pas de rafale, avec le temps accumulé pendant le rechargement.
    assert!(tirer_pendant(&mut etat, &arme, 0.95) <= 10);
}

#[test]
fn on_peut_recharger_avant_que_le_chargeur_soit_vide() {
    let arme = Arme::fusil_d_assaut();
    let mut etat = EtatArme::charge(&arme);
    tirer_pendant(&mut etat, &arme, 0.45);
    assert_eq!(etat.munitions, 25);

    actionner_l_arme(&mut etat, &arme, false, true, DT);
    // Pendant les 2 secondes du rechargement, moins un tick, on ne peut pas tirer...
    for _ in 0..(2.0 / DT) as u32 - 1 {
        assert!(!actionner_l_arme(&mut etat, &arme, true, false, DT));
    }
    // ... et au dernier tick, le rechargement se termine.
    actionner_l_arme(&mut etat, &arme, false, false, DT);

    assert_eq!(etat.munitions, 30);
    assert_eq!(etat.rechargement, None);
}

#[test]
fn recharger_un_chargeur_plein_ne_fait_rien() {
    let arme = Arme::fusil_d_assaut();
    let mut etat = EtatArme::charge(&arme);

    actionner_l_arme(&mut etat, &arme, false, true, DT);

    assert_eq!(etat.rechargement, None);
}

// Le plugin complet.

/// Une application avec les plugins des chapitres 11 à 13, un joueur armé à l'origine qui regarde
/// vers -Z, et une cible à 10 m devant lui.
fn app_avec_tireur_et_cible() -> (App, Entity, Entity) {
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
        ))
        .id();
    let cible = app
        .world_mut()
        .spawn((
            Physique {
                position: Vec3::new(0.0, 0.0, -10.0),
                au_sol: true,
                ..default()
            },
            Hitboxes::default(),
        ))
        .id();
    (app, joueur, cible)
}

fn messages<M: Message + Copy>(app: &App) -> Vec<M> {
    let messages = app.world().resource::<Messages<M>>();
    messages.get_cursor().read(messages).copied().collect()
}

#[test]
fn cliquer_tire_une_balle_dans_la_tete_de_la_cible() {
    let (mut app, joueur, cible) = app_avec_tireur_et_cible();

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    image(&mut app, TICK);

    let tirs = messages::<Tir>(&app);
    assert_eq!(tirs.len(), 1);
    assert_eq!(tirs[0].tireur, joueur);
    let touches = messages::<Touche>(&app);
    assert_eq!(touches.len(), 1);
    assert_eq!(touches[0].cible, cible);
    assert_eq!(touches[0].zone, Zone::Tete);
    assert!(proche(touches[0].degats, 60.0));
    assert_eq!(app.world().get::<EtatArme>(joueur).unwrap().munitions, 29);
}

#[test]
fn on_ne_tire_pas_en_sprintant() {
    let (mut app, _, _) = app_avec_tireur_et_cible();

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    let mut clavier = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    clavier.press(KeyCode::KeyW);
    clavier.press(KeyCode::ShiftLeft);
    image(&mut app, Duration::from_millis(100));

    assert!(messages::<Tir>(&app).is_empty());
}

#[test]
fn la_touche_r_recharge() {
    let (mut app, joueur, _) = app_avec_tireur_et_cible();
    app.world_mut()
        .get_mut::<EtatArme>(joueur)
        .unwrap()
        .munitions = 3;

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    image(&mut app, TICK);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyR);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    for _ in 0..21 {
        image(&mut app, Duration::from_millis(100));
    }

    assert_eq!(app.world().get::<EtatArme>(joueur).unwrap().munitions, 30);
}
