//! Solution du chapitre 8 : organiser le jeu en plugins, et ordonner les systèmes.

use bevy::prelude::*;

/// Les étapes d'une image de jeu, exécutées dans cet ordre par [`PluginJeu`].
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Etape {
    /// Lire le clavier.
    Entrees,
    /// Déplacer les entités.
    Deplacement,
    /// Détecter les collisions et leurs conséquences.
    Collisions,
}

/// Marque le joueur.
#[derive(Component)]
pub struct Joueur;

/// Marque les pièces à ramasser.
#[derive(Component)]
pub struct Piece;

/// La direction choisie par le joueur, de longueur 1 ou nulle.
#[derive(Resource, Default, Debug, PartialEq)]
pub struct Direction(pub Vec2);

/// La valeur d'une pièce, choisie dans [`PluginPieces`].
#[derive(Resource)]
pub struct ValeurPiece(pub u32);

/// Le score du joueur.
#[derive(Resource, Default)]
pub struct Score(pub u32);

/// Un message envoyé quand le joueur ramasse une pièce.
#[derive(Message)]
pub struct PieceRamassee;

/// La vitesse du joueur, en pixels par seconde.
pub const VITESSE_JOUEUR: f32 = 200.0;

/// La distance en dessous de laquelle le joueur ramasse une pièce, en pixels.
pub const RAYON_DE_RAMASSAGE: f32 = 32.0;

/// Écrit dans [`Direction`] la direction choisie avec les flèches du clavier.
pub fn lire_les_entrees(clavier: Res<ButtonInput<KeyCode>>, mut direction: ResMut<Direction>) {
    let mut nouvelle_direction = Vec2::ZERO;
    if clavier.pressed(KeyCode::ArrowUp) {
        nouvelle_direction.y += 1.0;
    }
    if clavier.pressed(KeyCode::ArrowDown) {
        nouvelle_direction.y -= 1.0;
    }
    if clavier.pressed(KeyCode::ArrowLeft) {
        nouvelle_direction.x -= 1.0;
    }
    if clavier.pressed(KeyCode::ArrowRight) {
        nouvelle_direction.x += 1.0;
    }
    direction.0 = nouvelle_direction.normalize_or_zero();
}

/// Déplace le joueur dans la [`Direction`] choisie, à [`VITESSE_JOUEUR`] pixels par seconde.
pub fn deplacer_le_joueur(
    temps: Res<Time>,
    direction: Res<Direction>,
    mut joueur: Single<&mut Transform, With<Joueur>>,
) {
    joueur.translation += (direction.0 * VITESSE_JOUEUR * temps.delta_secs()).extend(0.0);
}

/// Détruit les pièces proches du joueur, et envoie un message [`PieceRamassee`] pour chacune.
pub fn ramasser_les_pieces(
    mut commands: Commands,
    joueur: Single<&Transform, With<Joueur>>,
    pieces: Query<(Entity, &Transform), With<Piece>>,
    mut pieces_ramassees: MessageWriter<PieceRamassee>,
) {
    for (entite, transform) in &pieces {
        if transform.translation.distance(joueur.translation) < RAYON_DE_RAMASSAGE {
            commands.entity(entite).despawn();
            pieces_ramassees.write(PieceRamassee);
        }
    }
}

/// Ajoute la [`ValeurPiece`] au [`Score`] pour chaque pièce ramassée.
pub fn compter_les_points(
    mut pieces_ramassees: MessageReader<PieceRamassee>,
    valeur: Res<ValeurPiece>,
    mut score: ResMut<Score>,
) {
    for _ in pieces_ramassees.read() {
        score.0 += valeur.0;
    }
}

/// Le joueur : la lecture du clavier ([`Etape::Entrees`]) et le déplacement
/// ([`Etape::Deplacement`]).
pub struct PluginJoueur;

impl Plugin for PluginJoueur {
    fn build(&self, app: &mut App) {
        app.init_resource::<Direction>()
            .add_systems(Update, lire_les_entrees.in_set(Etape::Entrees))
            .add_systems(Update, deplacer_le_joueur.in_set(Etape::Deplacement));
    }
}

/// Les pièces : le ramassage et le score ([`Etape::Collisions`]). Chaque pièce rapporte `valeur`
/// points.
pub struct PluginPieces {
    /// Le nombre de points que rapporte chaque pièce.
    pub valeur: u32,
}

impl Plugin for PluginPieces {
    fn build(&self, app: &mut App) {
        app.add_message::<PieceRamassee>()
            .insert_resource(ValeurPiece(self.valeur))
            .init_resource::<Score>()
            .add_systems(
                Update,
                (ramasser_les_pieces, compter_les_points)
                    .chain()
                    .in_set(Etape::Collisions),
            );
    }
}

/// Le jeu complet : ordonne les étapes, et ajoute les plugins du joueur et des pièces (qui valent
/// 1 point chacune).
pub struct PluginJeu;

impl Plugin for PluginJeu {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (Etape::Entrees, Etape::Deplacement, Etape::Collisions).chain(),
        )
        .add_plugins((PluginJoueur, PluginPieces { valeur: 1 }));
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
