//! Solution du chapitre 3 : créer, modifier et détruire des entités avec les commandes.

use bevy::prelude::*;

/// Les points de vie d'une entité.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Vie(pub u32);

/// Marque les ennemis.
#[derive(Component, Default)]
pub struct Ennemi;

/// Un slime, le plus petit des ennemis.
///
/// Grâce aux composants requis, créer un `Slime` ajoute aussi [`Ennemi`], et [`Vie`] avec
/// 3 points de vie, s'ils ne sont pas déjà donnés.
#[derive(Component)]
#[require(Ennemi, Vie = Vie(3))]
pub struct Slime;

/// Un poison, qui retire 1 point de vie par image pendant `images_restantes` images.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Poison {
    /// Le nombre d'images pendant lesquelles le poison agit encore.
    pub images_restantes: u32,
}

/// Marque les ennemis enragés : ceux à qui il ne reste qu'un point de vie.
#[derive(Component)]
pub struct Enrage;

/// Le nombre de slimes à faire apparaître au début de la partie.
#[derive(Resource)]
pub struct TailleVague(pub u32);

/// Fait apparaître [`TailleVague`] slimes.
pub fn faire_apparaitre_la_vague(mut commands: Commands, taille: Res<TailleVague>) {
    for _ in 0..taille.0 {
        commands.spawn(Slime);
    }
}

/// Retire 1 point de vie à chaque entité empoisonnée, et diminue d'une image la durée de son
/// poison. Quand le poison est épuisé, il est retiré de l'entité.
///
/// Les points de vie ne descendent jamais sous 0.
pub fn appliquer_le_poison(
    mut commands: Commands,
    mut empoisonnes: Query<(Entity, &mut Vie, &mut Poison)>,
) {
    for (entite, mut vie, mut poison) in &mut empoisonnes {
        vie.0 = vie.0.saturating_sub(1);
        poison.images_restantes = poison.images_restantes.saturating_sub(1);
        if poison.images_restantes == 0 {
            commands.entity(entite).remove::<Poison>();
        }
    }
}

/// Ajoute le marqueur [`Enrage`] aux ennemis à qui il ne reste qu'un point de vie.
pub fn enrager_les_blesses(
    mut commands: Commands,
    blesses: Query<(Entity, &Vie), (With<Ennemi>, Without<Enrage>)>,
) {
    for (entite, vie) in &blesses {
        if vie.0 == 1 {
            commands.entity(entite).insert(Enrage);
        }
    }
}

/// Détruit les entités qui n'ont plus de points de vie.
pub fn retirer_les_morts(mut commands: Commands, entites: Query<(Entity, &Vie)>) {
    for (entite, vie) in &entites {
        if vie.0 == 0 {
            commands.entity(entite).despawn();
        }
    }
}

/// Le plugin du chapitre : la vague de 5 slimes au démarrage, puis à chaque image le poison, la
/// rage et la disparition des morts, dans cet ordre.
pub fn plugin(app: &mut App) {
    app.insert_resource(TailleVague(5))
        .add_systems(Startup, faire_apparaitre_la_vague)
        .add_systems(
            Update,
            (appliquer_le_poison, enrager_les_blesses, retirer_les_morts).chain(),
        );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
