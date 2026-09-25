//! Solution du chapitre 2 : les entités, les composants et les requêtes.

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
pub fn creer_le_monde(mut commands: Commands) {
    commands.spawn((
        Joueur,
        Nom("Héros".to_string()),
        Position(Vec2::ZERO),
        Vitesse(Vec2::new(1.0, 0.0)),
    ));
    commands.spawn((
        Ennemi,
        Nom("Gobelin".to_string()),
        Position(Vec2::new(10.0, 0.0)),
        Vitesse(Vec2::new(-1.0, 0.0)),
    ));
    commands.spawn((
        Ennemi,
        Nom("Troll".to_string()),
        Position(Vec2::new(-30.0, 5.0)),
    ));
    commands.spawn((Nom("Arbre".to_string()), Position(Vec2::new(5.0, 5.0))));
}

/// Ajoute la [`Vitesse`] de chaque entité à sa [`Position`].
pub fn deplacer(mut entites: Query<(&mut Position, &Vitesse)>) {
    for (mut position, vitesse) in &mut entites {
        position.0 += vitesse.0;
    }
}

/// Compte les ennemis, dans [`NombreEnnemis`].
pub fn compter_les_ennemis(ennemis: Query<(), With<Ennemi>>, mut nombre: ResMut<NombreEnnemis>) {
    nombre.0 = ennemis.iter().count();
}

/// Compte les décors, dans [`NombreDecors`].
pub fn compter_les_decors(
    decors: Query<(), (With<Position>, Without<Vitesse>, Without<Ennemi>)>,
    mut nombre: ResMut<NombreDecors>,
) {
    nombre.0 = decors.iter().count();
}

/// Écrit le nom de l'ennemi le plus proche du joueur dans la [`Cible`], ou `None` s'il n'y a pas
/// d'ennemi.
///
/// `Single` demande exactement une entité : s'il n'y a pas de joueur (ou s'il y en a plusieurs),
/// Bevy n'exécute pas ce système.
pub fn choisir_une_cible(
    joueur: Single<&Position, With<Joueur>>,
    ennemis: Query<(&Nom, &Position), With<Ennemi>>,
    mut cible: ResMut<Cible>,
) {
    let plus_proche = ennemis.iter().min_by(|(_, a), (_, b)| {
        let distance_a = a.0.distance(joueur.0);
        let distance_b = b.0.distance(joueur.0);
        distance_a.total_cmp(&distance_b)
    });
    cible.0 = plus_proche.map(|(nom, _)| nom.0.clone());
}

/// Le plugin du chapitre.
pub fn plugin(app: &mut App) {
    app.init_resource::<NombreEnnemis>()
        .init_resource::<NombreDecors>()
        .init_resource::<Cible>()
        .add_systems(Startup, creer_le_monde)
        .add_systems(
            Update,
            (
                deplacer,
                compter_les_ennemis,
                compter_les_decors,
                choisir_une_cible,
            )
                .chain(),
        );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
