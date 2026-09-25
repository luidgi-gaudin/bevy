//! Les tests du chapitre 6.

use super::*;
use bevy_tutorials::outils::compter;

/// Renvoie les messages [`PieceRamassee`] envoyés pendant les deux dernières images.
fn messages_envoyes(app: &App) -> Vec<PieceRamassee> {
    let messages = app.world().resource::<Messages<PieceRamassee>>();
    messages.get_cursor().read(messages).copied().collect()
}

/// Une application avec le message [`PieceRamassee`], le joueur au centre, et une pièce de
/// valeur 5 à la position donnée.
fn app_avec_une_piece(x: f32) -> App {
    let mut app = App::new();
    app.add_message::<PieceRamassee>();
    app.add_systems(Update, ramasser_les_pieces);
    app.world_mut().spawn((Joueur, Transform::default()));
    app.world_mut()
        .spawn((Piece { valeur: 5 }, Transform::from_xyz(x, 0.0, 0.0)));
    app
}

#[test]
fn ramasser_une_piece_proche_envoie_un_message() {
    let mut app = app_avec_une_piece(10.0);

    app.update();

    assert_eq!(compter::<Piece>(&mut app), 0);
    assert_eq!(messages_envoyes(&app), [PieceRamassee { valeur: 5 }]);
}

#[test]
fn une_piece_trop_loin_n_est_pas_ramassee() {
    let mut app = app_avec_une_piece(100.0);

    app.update();

    assert_eq!(compter::<Piece>(&mut app), 1);
    assert!(messages_envoyes(&app).is_empty());
}

#[test]
fn le_score_augmente_de_la_valeur_des_pieces() {
    let mut app = App::new();
    app.add_message::<PieceRamassee>();
    app.init_resource::<Score>();
    app.add_systems(Update, compter_les_points);

    // Un test peut envoyer des messages lui-même, sans système.
    let mut messages = app.world_mut().resource_mut::<Messages<PieceRamassee>>();
    messages.write(PieceRamassee { valeur: 3 });
    messages.write(PieceRamassee { valeur: 4 });
    app.update();

    assert_eq!(app.world().resource::<Score>().0, 7);
}

#[test]
fn chaque_lecteur_recoit_tous_les_messages() {
    let mut app = App::new();
    app.add_message::<PieceRamassee>();
    app.init_resource::<Score>();
    app.init_resource::<Journal>();
    app.add_systems(Update, (compter_les_points, tenir_le_journal));

    app.world_mut().write_message(PieceRamassee { valeur: 3 });
    app.world_mut().write_message(PieceRamassee { valeur: 4 });
    app.update();

    assert_eq!(app.world().resource::<Score>().0, 7);
    assert_eq!(app.world().resource::<Journal>().0, ["+3", "+4"]);
}

#[test]
fn un_message_n_est_lu_qu_une_fois_par_chaque_lecteur() {
    let mut app = App::new();
    app.add_message::<PieceRamassee>();
    app.init_resource::<Score>();
    app.add_systems(Update, compter_les_points);

    app.world_mut().write_message(PieceRamassee { valeur: 3 });
    app.update();
    app.update();
    app.update();

    assert_eq!(app.world().resource::<Score>().0, 3);
}

#[test]
fn toucher_une_entite_lui_retire_des_points_de_vie() {
    let mut app = App::new();
    app.add_observer(subir_les_coups);
    let ennemi = app.world_mut().spawn(Vie(3)).id();

    // Les observateurs s'exécutent immédiatement, pas besoin d'`update`.
    app.world_mut().trigger(Touche {
        entity: ennemi,
        degats: 1,
    });

    assert_eq!(app.world().get::<Vie>(ennemi), Some(&Vie(2)));
}

#[test]
fn un_ennemi_sans_vie_disparait_et_rapporte_des_points() {
    let mut app = App::new();
    app.init_resource::<Score>();
    app.add_observer(subir_les_coups);
    app.add_observer(recompenser);
    let ennemi = app.world_mut().spawn(Vie(2)).id();

    app.world_mut().trigger(Touche {
        entity: ennemi,
        degats: 5,
    });
    // Les commandes envoyées par les observateurs attendent le prochain `flush` (ou la prochaine
    // image) pour être appliquées.
    app.world_mut().flush();

    assert!(app.world().get_entity(ennemi).is_err());
    assert_eq!(app.world().resource::<Score>().0, POINTS_PAR_ENNEMI);
}

#[test]
fn toucher_une_entite_sans_vie_ne_fait_rien() {
    let mut app = App::new();
    app.init_resource::<Score>();
    app.add_observer(subir_les_coups);
    app.add_observer(recompenser);
    let rocher = app.world_mut().spawn(Transform::default()).id();

    app.world_mut().trigger(Touche {
        entity: rocher,
        degats: 5,
    });
    app.world_mut().flush();

    assert!(app.world().get_entity(rocher).is_ok());
    assert_eq!(app.world().resource::<Score>().0, 0);
}

#[test]
fn l_apparition_des_pieces_est_observee() {
    let mut app = App::new();
    app.init_resource::<PiecesApparues>();
    app.add_observer(compter_les_apparitions);

    app.world_mut().spawn(Piece { valeur: 1 });
    app.world_mut()
        .spawn((Piece { valeur: 2 }, Transform::default()));
    // Pas une pièce.
    app.world_mut().spawn(Transform::default());

    assert_eq!(app.world().resource::<PiecesApparues>().0, 2);
}

#[test]
fn le_plugin_assemble_tout() {
    let mut app = App::new();
    app.add_plugins(plugin);
    app.world_mut().spawn((Joueur, Transform::default()));
    app.world_mut()
        .spawn((Piece { valeur: 1 }, Transform::from_xyz(10.0, 0.0, 0.0)));
    app.world_mut()
        .spawn((Piece { valeur: 5 }, Transform::from_xyz(0.0, 20.0, 0.0)));
    app.world_mut()
        .spawn((Piece { valeur: 50 }, Transform::from_xyz(100.0, 0.0, 0.0)));
    let ennemi = app.world_mut().spawn(Vie(1)).id();

    app.update();

    assert_eq!(app.world().resource::<Score>().0, 6);
    assert_eq!(app.world().resource::<PiecesApparues>().0, 3);
    // L'ordre dans lequel une requête parcourt les entités n'est pas garanti : on trie.
    let mut journal = app.world().resource::<Journal>().0.clone();
    journal.sort();
    assert_eq!(journal, ["+1", "+5"]);

    app.world_mut().trigger(Touche {
        entity: ennemi,
        degats: 1,
    });
    app.update();

    assert_eq!(app.world().resource::<Score>().0, 6 + POINTS_PAR_ENNEMI);
}
