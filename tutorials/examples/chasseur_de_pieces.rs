//! « Chasseur de pièces », le jeu du projet final (chapitre 9) : ramassez un maximum de pièces en
//! 30 secondes !
//!
//! ```text
//! cargo run -p bevy_tutorials --example chasseur_de_pieces --features jeu
//! ```
//!
//! Toute la logique du jeu est dans `JeuPlugin`, testée sans fenêtre. Cet exemple n'ajoute que
//! l'affichage : une caméra, des carrés de couleur et du texte.
//!
//! Pour jouer à votre propre version (le fichier `src/chapitre_09/exercice.rs`), ajoutez la
//! fonctionnalité `exercice` : `--features jeu,exercice`.

use bevy::prelude::*;
use bevy_tutorials::affichage::police_avec_accents;

#[cfg(not(feature = "exercice"))]
use bevy_tutorials::chapitre_09::solution as jeu;

// Votre version du jeu, qui contient peut-être encore des `todo!()`.
#[cfg(feature = "exercice")]
#[allow(unused_variables, unused_mut, unused_imports, dead_code)]
#[path = "../src/chapitre_09/exercice.rs"]
mod jeu;

use jeu::{
    ChronoPartie, EtatJeu, JeuPlugin, Joueur, MeilleurScore, Piece, Score, DEMI_HAUTEUR_ARENE,
    DEMI_LARGEUR_ARENE, TAILLE_JOUEUR, TAILLE_PIECE,
};

/// Marque le texte qui affiche le score et les instructions.
#[derive(Component)]
struct TexteInfos;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Chasseur de pièces".to_string(),
                    resolution: (800, 600).into(),
                    ..default()
                }),
                ..default()
            }),
            // La police par défaut de Bevy n'a pas les accents du français.
            police_avec_accents,
            JeuPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.1)))
        .add_systems(Startup, installer_la_scene)
        // Les observateurs donnent une apparence aux entités créées par la logique du jeu.
        .add_observer(dessiner_le_joueur)
        .add_observer(dessiner_les_pieces)
        .add_systems(Update, afficher_les_infos)
        .run();
}

/// Crée la caméra, le fond de l'arène et le texte.
fn installer_la_scene(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.12, 0.15, 0.22),
            Vec2::new(2.0 * DEMI_LARGEUR_ARENE, 2.0 * DEMI_HAUTEUR_ARENE),
        ),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
    commands.spawn((
        TexteInfos,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

/// Le joueur est un carré bleu.
fn dessiner_le_joueur(ajout: On<Add<Joueur>>, mut commands: Commands) {
    commands.entity(ajout.entity).insert(Sprite::from_color(
        Color::srgb(0.3, 0.7, 1.0),
        Vec2::splat(TAILLE_JOUEUR),
    ));
}

/// Les pièces sont des carrés dorés.
fn dessiner_les_pieces(ajout: On<Add<Piece>>, mut commands: Commands) {
    commands.entity(ajout.entity).insert(Sprite::from_color(
        Color::srgb(1.0, 0.8, 0.2),
        Vec2::splat(TAILLE_PIECE),
    ));
}

/// Affiche les instructions, le score et le temps restant.
fn afficher_les_infos(
    etat: Res<State<EtatJeu>>,
    score: Res<Score>,
    meilleur_score: Res<MeilleurScore>,
    chrono: Res<ChronoPartie>,
    mut texte: Single<&mut Text, With<TexteInfos>>,
) {
    texte.0 = match etat.get() {
        EtatJeu::Menu => "Chasseur de pièces\n\n\
            Ramassez un maximum de pièces en 30 secondes !\n\
            Flèches ou ZQSD : se déplacer\n\n\
            Entrée : jouer"
            .to_string(),
        EtatJeu::EnJeu => format!(
            "Score : {}\nTemps : {:.0} s",
            score.0,
            chrono.0.remaining_secs().ceil()
        ),
        EtatJeu::FinDePartie => format!(
            "Partie terminée !\n\nScore : {}\nMeilleur score : {}\n\nEntrée : rejouer",
            score.0, meilleur_score.0
        ),
    };
}
