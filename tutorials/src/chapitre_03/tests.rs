//! Les tests du chapitre 3.

use super::*;
use bevy_tutorials::outils::compter;

#[test]
fn la_vague_fait_apparaitre_des_slimes() {
    let mut app = App::new();
    app.insert_resource(TailleVague(4));
    app.add_systems(Startup, faire_apparaitre_la_vague);

    app.update();

    assert_eq!(compter::<Slime>(&mut app), 4);
    // Les composants requis ont été ajoutés automatiquement.
    assert_eq!(compter::<Ennemi>(&mut app), 4);
    let vies: Vec<Vie> = app
        .world_mut()
        .query::<&Vie>()
        .iter(app.world())
        .copied()
        .collect();
    assert_eq!(vies, [Vie(3); 4]);
}

#[test]
fn un_slime_est_un_ennemi_avec_trois_points_de_vie() {
    let mut world = World::new();

    let slime = world.spawn(Slime).id();

    assert!(world.get::<Ennemi>(slime).is_some());
    assert_eq!(world.get::<Vie>(slime), Some(&Vie(3)));
}

#[test]
fn les_composants_donnes_remplacent_les_composants_requis() {
    let mut world = World::new();

    let gros_slime = world.spawn((Slime, Vie(10))).id();

    assert_eq!(world.get::<Vie>(gros_slime), Some(&Vie(10)));
    assert!(world.get::<Ennemi>(gros_slime).is_some());
}

#[test]
fn le_poison_retire_un_point_de_vie_par_image() {
    let mut app = App::new();
    app.add_systems(Update, appliquer_le_poison);
    let slime = app
        .world_mut()
        .spawn((
            Vie(5),
            Poison {
                images_restantes: 10,
            },
        ))
        .id();

    app.update();
    app.update();

    assert_eq!(app.world().get::<Vie>(slime), Some(&Vie(3)));
    assert_eq!(
        app.world().get::<Poison>(slime),
        Some(&Poison {
            images_restantes: 8
        })
    );
}

#[test]
fn le_poison_est_retire_quand_il_est_epuise() {
    let mut app = App::new();
    app.add_systems(Update, appliquer_le_poison);
    let slime = app
        .world_mut()
        .spawn((
            Vie(5),
            Poison {
                images_restantes: 2,
            },
        ))
        .id();

    app.update();
    app.update();

    assert_eq!(app.world().get::<Vie>(slime), Some(&Vie(3)));
    assert!(app.world().get::<Poison>(slime).is_none());

    // Sans poison, la vie ne baisse plus.
    app.update();
    assert_eq!(app.world().get::<Vie>(slime), Some(&Vie(3)));
}

#[test]
fn la_vie_ne_descend_pas_sous_zero() {
    let mut app = App::new();
    app.add_systems(Update, appliquer_le_poison);
    let slime = app
        .world_mut()
        .spawn((
            Vie(1),
            Poison {
                images_restantes: 5,
            },
        ))
        .id();

    app.update();
    app.update();

    assert_eq!(app.world().get::<Vie>(slime), Some(&Vie(0)));
}

#[test]
fn les_ennemis_blesses_deviennent_enrages() {
    let mut app = App::new();
    app.add_systems(Update, enrager_les_blesses);
    let blesse = app.world_mut().spawn((Ennemi, Vie(1))).id();
    let en_forme = app.world_mut().spawn((Ennemi, Vie(3))).id();
    // Pas un ennemi : ne s'enrage pas.
    let villageois = app.world_mut().spawn(Vie(1)).id();

    app.update();

    assert!(app.world().get::<Enrage>(blesse).is_some());
    assert!(app.world().get::<Enrage>(en_forme).is_none());
    assert!(app.world().get::<Enrage>(villageois).is_none());
}

#[test]
fn les_entites_sans_vie_disparaissent() {
    let mut app = App::new();
    app.add_systems(Update, retirer_les_morts);
    let mort = app.world_mut().spawn(Vie(0)).id();
    let vivant = app.world_mut().spawn(Vie(2)).id();

    app.update();

    // Une entité détruite n'existe plus du tout.
    assert!(app.world().get_entity(mort).is_err());
    assert!(app.world().get_entity(vivant).is_ok());
}

#[test]
fn un_slime_empoisonne_finit_par_disparaitre() {
    let mut app = App::new();
    app.add_plugins(plugin);
    // La première image fait apparaître la vague.
    app.update();
    assert_eq!(compter::<Slime>(&mut app), 5);

    let slime = app
        .world_mut()
        .spawn((
            Slime,
            Poison {
                images_restantes: 5,
            },
        ))
        .id();

    app.update();
    app.update();
    // Il ne lui reste qu'un point de vie : il s'enrage.
    assert!(app.world().get::<Enrage>(slime).is_some());

    // À la troisième image, il meurt, et disparaît dans la même image grâce à l'ordre des
    // systèmes.
    app.update();
    assert!(app.world().get_entity(slime).is_err());
    assert_eq!(compter::<Slime>(&mut app), 5);
}
