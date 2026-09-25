//! Solution du chapitre 6 : communiquer entre systèmes avec les messages et les observateurs.

use bevy::prelude::*;

/// Marque le joueur.
#[derive(Component)]
pub struct Joueur;

/// Une pièce à ramasser, qui rapporte `valeur` points.
#[derive(Component)]
pub struct Piece {
    /// Le nombre de points que rapporte la pièce.
    pub valeur: u32,
}

/// Les points de vie d'une entité.
#[derive(Component, Debug, PartialEq)]
pub struct Vie(pub u32);

/// Le score du joueur.
#[derive(Resource, Default)]
pub struct Score(pub u32);

/// Les événements marquants de la partie, du plus ancien au plus récent.
#[derive(Resource, Default)]
pub struct Journal(pub Vec<String>);

/// Le nombre de pièces apparues depuis le début de la partie.
#[derive(Resource, Default)]
pub struct PiecesApparues(pub u32);

/// La distance en dessous de laquelle le joueur ramasse une pièce, en pixels.
pub const RAYON_DE_RAMASSAGE: f32 = 32.0;

/// Les points gagnés pour chaque ennemi vaincu.
pub const POINTS_PAR_ENNEMI: u32 = 10;

/// Un message envoyé quand le joueur ramasse une pièce.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct PieceRamassee {
    /// La valeur de la pièce ramassée.
    pub valeur: u32,
}

/// Un coup porté à une entité : un événement qui vise cette entité.
#[derive(EntityEvent)]
pub struct Touche {
    /// L'entité touchée.
    pub entity: Entity,
    /// Le nombre de points de vie perdus.
    pub degats: u32,
}

/// Un événement déclenché quand un ennemi est vaincu.
#[derive(Event)]
pub struct EnnemiVaincu;

/// Détruit les pièces proches du joueur (à moins de [`RAYON_DE_RAMASSAGE`] pixels), et envoie un
/// message [`PieceRamassee`] pour chacune.
pub fn ramasser_les_pieces(
    mut commands: Commands,
    joueur: Single<&Transform, With<Joueur>>,
    pieces: Query<(Entity, &Transform, &Piece)>,
    mut pieces_ramassees: MessageWriter<PieceRamassee>,
) {
    for (entite, transform, piece) in &pieces {
        if transform.translation.distance(joueur.translation) < RAYON_DE_RAMASSAGE {
            commands.entity(entite).despawn();
            pieces_ramassees.write(PieceRamassee {
                valeur: piece.valeur,
            });
        }
    }
}

/// Ajoute la valeur des pièces ramassées au [`Score`].
pub fn compter_les_points(
    mut pieces_ramassees: MessageReader<PieceRamassee>,
    mut score: ResMut<Score>,
) {
    for piece in pieces_ramassees.read() {
        score.0 += piece.valeur;
    }
}

/// Écrit `"+<valeur>"` dans le [`Journal`] pour chaque pièce ramassée.
pub fn tenir_le_journal(
    mut pieces_ramassees: MessageReader<PieceRamassee>,
    mut journal: ResMut<Journal>,
) {
    for piece in pieces_ramassees.read() {
        journal.0.push(format!("+{}", piece.valeur));
    }
}

/// Un observateur de [`Touche`] : retire les dégâts à la [`Vie`] de l'entité touchée. Si elle n'a
/// plus de points de vie, la détruit et déclenche [`EnnemiVaincu`].
pub fn subir_les_coups(touche: On<Touche>, mut vies: Query<&mut Vie>, mut commands: Commands) {
    let Ok(mut vie) = vies.get_mut(touche.entity) else {
        return;
    };
    vie.0 = vie.0.saturating_sub(touche.degats);
    if vie.0 == 0 {
        commands.entity(touche.entity).despawn();
        commands.trigger(EnnemiVaincu);
    }
}

/// Un observateur de [`EnnemiVaincu`] : ajoute [`POINTS_PAR_ENNEMI`] au [`Score`].
pub fn recompenser(_vaincu: On<EnnemiVaincu>, mut score: ResMut<Score>) {
    score.0 += POINTS_PAR_ENNEMI;
}

/// Un observateur de l'ajout du composant [`Piece`] à une entité : compte les pièces apparues.
pub fn compter_les_apparitions(_ajout: On<Add<Piece>>, mut apparues: ResMut<PiecesApparues>) {
    apparues.0 += 1;
}

/// Le plugin du chapitre.
pub fn plugin(app: &mut App) {
    app.add_message::<PieceRamassee>()
        .init_resource::<Score>()
        .init_resource::<Journal>()
        .init_resource::<PiecesApparues>()
        .add_systems(
            Update,
            (
                ramasser_les_pieces,
                // Les deux lecteurs après l'écrivain, pour lire les messages dans la même image.
                (compter_les_points, tenir_le_journal),
            )
                .chain(),
        )
        .add_observer(subir_les_coups)
        .add_observer(recompenser)
        .add_observer(compter_les_apparitions);
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
