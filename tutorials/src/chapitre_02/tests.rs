//! Les tests du chapitre 2.

use super::*;
use bevy_tutorials::outils::compter;

/// Renvoie les noms de toutes les entités, triés par ordre alphabétique.
fn noms(app: &mut App) -> Vec<String> {
    let mut noms: Vec<String> = app
        .world_mut()
        .query::<&Nom>()
        .iter(app.world())
        .map(|nom| nom.0.clone())
        .collect();
    noms.sort();
    noms
}

#[test]
fn le_monde_contient_un_joueur_des_ennemis_et_un_decor() {
    let mut app = App::new();
    app.add_systems(Startup, creer_le_monde);
    app.update();

    assert_eq!(noms(&mut app), ["Arbre", "Gobelin", "Héros", "Troll"]);
    assert_eq!(compter::<Joueur>(&mut app), 1);
    assert_eq!(compter::<Ennemi>(&mut app), 2);
    assert_eq!(compter::<Vitesse>(&mut app), 2);
}

#[test]
fn les_entites_avancent_selon_leur_vitesse() {
    let mut app = App::new();
    app.add_systems(Update, deplacer);
    // On peut créer des entités directement dans le monde, pour préparer un test.
    let entite = app
        .world_mut()
        .spawn((Position(Vec2::new(1.0, 2.0)), Vitesse(Vec2::new(3.0, -1.0))))
        .id();

    app.update();
    app.update();

    let position = app.world().get::<Position>(entite).unwrap();
    assert_eq!(position.0, Vec2::new(7.0, 0.0));
}

#[test]
fn les_entites_sans_vitesse_ne_bougent_pas() {
    let mut app = App::new();
    app.add_systems(Update, deplacer);
    let arbre = app.world_mut().spawn(Position(Vec2::new(5.0, 5.0))).id();

    app.update();

    assert_eq!(
        app.world().get::<Position>(arbre),
        Some(&Position(Vec2::new(5.0, 5.0)))
    );
}

#[test]
fn on_compte_les_ennemis() {
    let mut app = App::new();
    app.init_resource::<NombreEnnemis>();
    app.add_systems(Update, compter_les_ennemis);
    app.world_mut().spawn(Ennemi);
    app.world_mut().spawn((Ennemi, Nom("Gobelin".to_string())));
    app.world_mut().spawn(Joueur);

    app.update();
    assert_eq!(app.world().resource::<NombreEnnemis>().0, 2);

    // Le compte est mis à jour à chaque image.
    app.world_mut().spawn(Ennemi);
    app.update();
    assert_eq!(app.world().resource::<NombreEnnemis>().0, 3);
}

#[test]
fn les_decors_sont_immobiles_et_ne_sont_pas_des_ennemis() {
    let mut app = App::new();
    app.init_resource::<NombreDecors>();
    app.add_systems(Update, compter_les_decors);
    // Deux décors.
    app.world_mut().spawn(Position(Vec2::ZERO));
    app.world_mut()
        .spawn((Nom("Rocher".to_string()), Position(Vec2::ONE)));
    // Pas des décors : il bouge, c'est un ennemi, il n'a pas de position.
    app.world_mut()
        .spawn((Position(Vec2::ZERO), Vitesse(Vec2::X)));
    app.world_mut().spawn((Ennemi, Position(Vec2::ZERO)));
    app.world_mut().spawn(Nom("Fantôme".to_string()));

    app.update();

    assert_eq!(app.world().resource::<NombreDecors>().0, 2);
}

#[test]
fn la_cible_est_l_ennemi_le_plus_proche() {
    let mut app = App::new();
    app.init_resource::<Cible>();
    app.add_systems(Update, choisir_une_cible);
    app.world_mut()
        .spawn((Joueur, Position(Vec2::new(0.0, 0.0))));
    app.world_mut().spawn((
        Ennemi,
        Nom("Loin".to_string()),
        Position(Vec2::new(0.0, 50.0)),
    ));
    app.world_mut().spawn((
        Ennemi,
        Nom("Proche".to_string()),
        Position(Vec2::new(-3.0, 4.0)),
    ));
    // Pas un ennemi : ne doit pas être choisi, même s'il est plus proche.
    app.world_mut()
        .spawn((Nom("Arbre".to_string()), Position(Vec2::new(1.0, 0.0))));

    app.update();

    assert_eq!(app.world().resource::<Cible>().0.as_deref(), Some("Proche"));
}

#[test]
fn sans_ennemi_il_n_y_a_pas_de_cible() {
    let mut app = App::new();
    app.insert_resource(Cible(Some("Ancienne cible".to_string())));
    app.add_systems(Update, choisir_une_cible);
    app.world_mut().spawn((Joueur, Position(Vec2::ZERO)));

    app.update();

    assert_eq!(app.world().resource::<Cible>().0, None);
}

#[test]
fn sans_joueur_la_cible_n_est_pas_choisie() {
    let mut app = App::new();
    app.init_resource::<Cible>();
    app.add_systems(Update, choisir_une_cible);
    app.world_mut()
        .spawn((Ennemi, Nom("Gobelin".to_string()), Position(Vec2::ZERO)));

    // Sans joueur, `Single` empêche le système de s'exécuter : pas d'erreur, pas de cible.
    app.update();

    assert_eq!(app.world().resource::<Cible>().0, None);
}

#[test]
fn le_plugin_assemble_tout() {
    let mut app = App::new();
    app.add_plugins(plugin);

    app.update();

    assert_eq!(app.world().resource::<NombreEnnemis>().0, 2);
    assert_eq!(app.world().resource::<NombreDecors>().0, 1);
    // Après une image, le héros est en (1, 0) et le gobelin en (9, 0) : il est à 8 pixels, le
    // troll à plus de 30.
    assert_eq!(
        app.world().resource::<Cible>().0.as_deref(),
        Some("Gobelin")
    );
}
