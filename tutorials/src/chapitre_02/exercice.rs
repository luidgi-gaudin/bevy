//! Exercice du chapitre 2 : les entités, les composants et les requêtes.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_02
//! ```
//!
//! Vous pouvez ajouter des paramètres aux systèmes : c'est même nécessaire pour la plupart d'entre
//! eux, qui ont besoin d'une requête (`Query`).

use bevy::prelude::*;

/// La position d'une entité.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Position(pub Vec2);

/// De combien une entité se déplace à chaque image.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Vitesse(pub Vec2);

/// Le nom d'une entité.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Nom(pub String);

/// Marque l'entité du joueur.
#[derive(Component)]
pub struct Joueur;

/// Marque les ennemis.
#[derive(Component)]
pub struct Ennemi;

/// Le nombre d'ennemis.
#[derive(Resource, Default)]
pub struct NombreEnnemis(pub usize);

/// Le nombre de décors : les entités qui ont une [`Position`], mais pas de [`Vitesse`], et qui ne
/// sont pas des ennemis.
#[derive(Resource, Default)]
pub struct NombreDecors(pub usize);

/// Le nom de l'ennemi le plus proche du joueur, s'il y a des ennemis.
#[derive(Resource, Default)]
pub struct Cible(pub Option<String>);

/// Crée le monde du jeu :
/// - le joueur, `"Héros"`, en (0, 0), avec une vitesse de (1, 0) ;
/// - un ennemi, `"Gobelin"`, en (10, 0), avec une vitesse de (-1, 0) ;
/// - un ennemi immobile, `"Troll"`, en (-30, 5) ;
/// - un décor, `"Arbre"`, en (5, 5).
///
/// Indice : `commands.spawn((Joueur, Nom("Héros".to_string()), ...))` crée une entité avec tous
/// les composants du tuple.
pub fn creer_le_monde(mut commands: Commands) {
    todo!()
}

/// Ajoute la [`Vitesse`] de chaque entité à sa [`Position`].
///
/// Indice : une requête `Query<(&mut Position, &Vitesse)>` donne toutes les entités qui ont ces
/// deux composants.
pub fn deplacer() {
    todo!()
}

/// Compte les ennemis, dans [`NombreEnnemis`].
///
/// Indice : `Query<(), With<Ennemi>>` donne les entités qui ont le composant `Ennemi`, sans lire
/// aucun composant.
pub fn compter_les_ennemis(mut nombre: ResMut<NombreEnnemis>) {
    todo!()
}

/// Compte les décors, dans [`NombreDecors`].
///
/// Indice : les filtres se combinent dans un tuple : `(With<A>, Without<B>)`.
pub fn compter_les_decors(mut nombre: ResMut<NombreDecors>) {
    todo!()
}

/// Écrit le nom de l'ennemi le plus proche du joueur dans la [`Cible`], ou `None` s'il n'y a pas
/// d'ennemi.
///
/// Indices :
/// - `Single<&Position, With<Joueur>>` donne la position de l'unique joueur. S'il n'y a pas de
///   joueur, Bevy n'exécute pas le système.
/// - `a.distance(b)` donne la distance entre deux `Vec2`.
/// - les itérateurs ont une méthode `min_by`, et `f32::total_cmp` compare deux `f32`.
pub fn choisir_une_cible(mut cible: ResMut<Cible>) {
    todo!()
}

/// Le plugin du chapitre : les trois ressources, [`creer_le_monde`] au démarrage, et les autres
/// systèmes à chaque image, dans l'ordre où ils sont écrits dans ce fichier.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
