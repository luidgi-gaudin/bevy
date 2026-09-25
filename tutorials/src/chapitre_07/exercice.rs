//! Exercice du chapitre 7 : les états du jeu (menu, partie, fin de partie).
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_07
//! ```

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
///
/// Indice : `ResMut<NextState<EtatJeu>>` permet de demander un changement d'état avec `set`.
pub fn lancer_la_partie(clavier: Res<ButtonInput<KeyCode>>) {
    todo!()
}

/// Au début de chaque partie : fait apparaître le joueur, qui disparaîtra à la fin de la partie,
/// remet le [`TempsRestant`] à [`DUREE_PARTIE`] et compte la partie dans [`PartiesJouees`].
///
/// Indices :
/// - le composant `DespawnOnExit(EtatJeu::EnJeu)` détruit l'entité quand on quitte cet état ;
/// - `commands.insert_resource(...)` ajoute ou remplace une ressource.
pub fn preparer_la_partie(mut commands: Commands) {
    todo!()
}

/// Pendant la partie : décompte le temps, et termine la partie quand il est écoulé. Le temps
/// restant ne descend pas sous 0.
pub fn decompter_le_temps(temps: Res<Time>, mut restant: ResMut<TempsRestant>) {
    todo!()
}

/// À la fin de la partie : revient au menu quand la touche Entrée vient d'être enfoncée.
pub fn revenir_au_menu(clavier: Res<ButtonInput<KeyCode>>) {
    todo!()
}

/// Le plugin du chapitre : l'état, la ressource [`PartiesJouees`], [`preparer_la_partie`] en
/// entrant dans [`EtatJeu::EnJeu`], et les trois autres systèmes à chaque image, chacun
/// seulement dans son état.
///
/// Indices : `app.init_state::<EtatJeu>()`, la planification `OnEnter(EtatJeu::EnJeu)`, et
/// `systeme.run_if(in_state(EtatJeu::Menu))`.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
