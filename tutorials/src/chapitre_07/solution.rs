//! Solution du chapitre 7 : les états du jeu (menu, partie, fin de partie).

use bevy::prelude::*;

/// Les états du jeu.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EtatJeu {
    /// L'écran titre, au lancement du jeu.
    #[default]
    Menu,
    /// Une partie est en cours.
    EnJeu,
    /// La partie est terminée.
    FinDePartie,
}

/// Marque le joueur.
#[derive(Component)]
pub struct Joueur;

/// Le temps qu'il reste avant la fin de la partie, en secondes.
#[derive(Resource, Debug, PartialEq)]
pub struct TempsRestant(pub f32);

/// Le nombre de parties commencées depuis le lancement du jeu.
#[derive(Resource, Default)]
pub struct PartiesJouees(pub u32);

/// La durée d'une partie, en secondes.
pub const DUREE_PARTIE: f32 = 30.0;

/// Dans le menu : lance la partie quand la touche Entrée vient d'être enfoncée.
pub fn lancer_la_partie(
    clavier: Res<ButtonInput<KeyCode>>,
    mut prochain_etat: ResMut<NextState<EtatJeu>>,
) {
    if clavier.just_pressed(KeyCode::Enter) {
        prochain_etat.set(EtatJeu::EnJeu);
    }
}

/// Au début de chaque partie : fait apparaître le joueur, qui disparaîtra à la fin de la partie,
/// remet le [`TempsRestant`] à [`DUREE_PARTIE`] et compte la partie dans [`PartiesJouees`].
pub fn preparer_la_partie(mut commands: Commands, mut parties: ResMut<PartiesJouees>) {
    commands.spawn((Joueur, DespawnOnExit(EtatJeu::EnJeu)));
    commands.insert_resource(TempsRestant(DUREE_PARTIE));
    parties.0 += 1;
}

/// Pendant la partie : décompte le temps, et termine la partie quand il est écoulé.
pub fn decompter_le_temps(
    temps: Res<Time>,
    mut restant: ResMut<TempsRestant>,
    mut prochain_etat: ResMut<NextState<EtatJeu>>,
) {
    restant.0 -= temps.delta_secs();
    if restant.0 <= 0.0 {
        restant.0 = 0.0;
        prochain_etat.set(EtatJeu::FinDePartie);
    }
}

/// À la fin de la partie : revient au menu quand la touche Entrée vient d'être enfoncée.
pub fn revenir_au_menu(
    clavier: Res<ButtonInput<KeyCode>>,
    mut prochain_etat: ResMut<NextState<EtatJeu>>,
) {
    if clavier.just_pressed(KeyCode::Enter) {
        prochain_etat.set(EtatJeu::Menu);
    }
}

/// Le plugin du chapitre. Il a besoin du `StatesPlugin`, qui fait partie de `DefaultPlugins`.
pub fn plugin(app: &mut App) {
    app.init_state::<EtatJeu>()
        .init_resource::<PartiesJouees>()
        .add_systems(OnEnter(EtatJeu::EnJeu), preparer_la_partie)
        .add_systems(
            Update,
            (
                lancer_la_partie.run_if(in_state(EtatJeu::Menu)),
                decompter_le_temps.run_if(in_state(EtatJeu::EnJeu)),
                revenir_au_menu.run_if(in_state(EtatJeu::FinDePartie)),
            ),
        );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
