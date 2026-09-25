//! Les tests du chapitre 1.
//!
//! Chaque test crée sa propre application, y ajoute ce qu'il veut vérifier, exécute quelques
//! images avec `app.update()`, puis lit le résultat dans le monde (`app.world()`).

use super::*;

#[test]
fn le_message_de_bienvenue_est_ecrit_au_demarrage() {
    let mut app = App::new();
    app.init_resource::<Bienvenue>();
    app.add_systems(Startup, ecrire_bienvenue);

    // La première image exécute d'abord les systèmes de démarrage.
    app.update();

    assert_eq!(
        app.world().resource::<Bienvenue>().0,
        "Bienvenue dans Bevy !"
    );
}

#[test]
fn les_systemes_de_demarrage_ne_s_executent_qu_une_fois() {
    let mut app = App::new();
    app.init_resource::<Bienvenue>();
    app.add_systems(Startup, ecrire_bienvenue);
    app.update();

    // On modifie la ressource nous-mêmes...
    app.world_mut().resource_mut::<Bienvenue>().0 = "Au revoir".to_string();
    app.update();

    // ... et le système de démarrage ne l'a pas réécrite.
    assert_eq!(app.world().resource::<Bienvenue>().0, "Au revoir");
}

#[test]
fn le_compteur_augmente_a_chaque_image() {
    let mut app = App::new();
    app.init_resource::<CompteurImages>();
    app.add_systems(Update, compter_les_images);

    for _ in 0..5 {
        app.update();
    }

    assert_eq!(app.world().resource::<CompteurImages>().0, 5);
}

#[test]
fn un_palier_est_annonce_toutes_les_10_images() {
    let mut app = App::new();
    app.insert_resource(CompteurImages(9));
    app.init_resource::<Journal>();
    app.add_systems(Update, annoncer_les_paliers);

    // 9 images : pas de palier.
    app.update();
    assert!(app.world().resource::<Journal>().0.is_empty());

    // 10 images : le palier est annoncé.
    app.world_mut().resource_mut::<CompteurImages>().0 = 10;
    app.update();
    assert_eq!(app.world().resource::<Journal>().0, ["10 images"]);
}

#[test]
fn aucun_palier_n_est_annonce_avant_la_premiere_image() {
    let mut app = App::new();
    app.init_resource::<CompteurImages>();
    app.init_resource::<Journal>();
    app.add_systems(Update, annoncer_les_paliers);

    app.update();

    assert!(app.world().resource::<Journal>().0.is_empty());
}

#[test]
fn le_plugin_assemble_tout() {
    let mut app = App::new();
    app.add_plugins(plugin);

    for _ in 0..25 {
        app.update();
    }

    assert_eq!(
        app.world().resource::<Bienvenue>().0,
        "Bienvenue dans Bevy !"
    );
    assert_eq!(app.world().resource::<CompteurImages>().0, 25);
    assert_eq!(
        app.world().resource::<Journal>().0,
        ["10 images", "20 images"]
    );
}
