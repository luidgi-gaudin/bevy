//! Solution du chapitre 9 : le projet final, « Chasseur de pièces ».
//!
//! Le joueur a 30 secondes pour ramasser un maximum de pièces, qui apparaissent au hasard dans
//! l'arène. Ce fichier ne contient que la logique du jeu, testée sans fenêtre : l'affichage est
//! ajouté par l'exemple `chasseur_de_pieces`.

use bevy::{prelude::*, state::app::StatesPlugin};

/// La moitié de la largeur de l'arène, en pixels. L'arène est centrée sur l'origine.
pub const DEMI_LARGEUR_ARENE: f32 = 400.0;

/// La moitié de la hauteur de l'arène, en pixels.
pub const DEMI_HAUTEUR_ARENE: f32 = 300.0;

/// La taille du joueur (un carré), en pixels.
pub const TAILLE_JOUEUR: f32 = 40.0;

/// La taille des pièces, en pixels.
pub const TAILLE_PIECE: f32 = 20.0;

/// La vitesse du joueur, en pixels par seconde.
pub const VITESSE_JOUEUR: f32 = 300.0;

/// La distance en dessous de laquelle le joueur ramasse une pièce, en pixels.
pub const RAYON_DE_RAMASSAGE: f32 = 30.0;

/// La durée d'une partie, en secondes.
pub const DUREE_PARTIE: f32 = 30.0;

/// Le temps entre deux apparitions de pièces, en secondes.
pub const INTERVALLE_PIECES: f32 = 1.0;

/// Le nombre maximal de pièces dans l'arène.
pub const PIECES_MAX: usize = 5;

/// Les états du jeu.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EtatJeu {
    /// L'écran titre.
    #[default]
    Menu,
    /// Une partie est en cours.
    EnJeu,
    /// La partie est terminée : le score est affiché.
    FinDePartie,
}

/// Marque le joueur.
#[derive(Component)]
pub struct Joueur;

/// Marque les pièces.
#[derive(Component)]
pub struct Piece;

/// Le score de la partie en cours (ou de la dernière partie).
#[derive(Resource, Default)]
pub struct Score(pub u32);

/// Le meilleur score depuis le lancement du jeu.
#[derive(Resource, Default)]
pub struct MeilleurScore(pub u32);

/// Le temps de la partie : elle se termine quand ce minuteur est terminé.
#[derive(Resource)]
pub struct ChronoPartie(pub Timer);

impl Default for ChronoPartie {
    fn default() -> Self {
        Self(Timer::from_seconds(DUREE_PARTIE, TimerMode::Once))
    }
}

/// Le minuteur qui fait apparaître les pièces.
#[derive(Resource)]
pub struct MinuteurPieces(pub Timer);

impl Default for MinuteurPieces {
    fn default() -> Self {
        Self(Timer::from_seconds(INTERVALLE_PIECES, TimerMode::Repeating))
    }
}

/// Un message envoyé quand le joueur ramasse une pièce.
#[derive(Message)]
pub struct PieceRamassee;

/// Un générateur de nombres pseudo-aléatoires (*xorshift*), simple et reproductible : avec la
/// même graine, il donne toujours la même suite de nombres, ce qui rend les tests fiables.
///
/// La graine (le nombre de départ) ne doit pas être nulle.
#[derive(Resource)]
pub struct Hasard(pub u32);

impl Default for Hasard {
    fn default() -> Self {
        Self(0x2545_F491)
    }
}

impl Hasard {
    /// Renvoie un nombre entre 0 (inclus) et 1 (exclus).
    pub fn nombre(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        // Les 24 bits de poids fort donnent un `f32` exact entre 0 et 1.
        (x >> 8) as f32 / (1 << 24) as f32
    }

    /// Renvoie une position au hasard dans l'arène, à au moins [`TAILLE_PIECE`] pixels des bords.
    pub fn position_dans_l_arene(&mut self) -> Vec2 {
        Vec2::new(
            (self.nombre() * 2.0 - 1.0) * (DEMI_LARGEUR_ARENE - TAILLE_PIECE),
            (self.nombre() * 2.0 - 1.0) * (DEMI_HAUTEUR_ARENE - TAILLE_PIECE),
        )
    }
}

/// Renvoie la direction choisie avec les flèches ou les touches WASD (ZQSD en AZERTY), de
/// longueur 1 ou nulle.
pub fn direction(clavier: &ButtonInput<KeyCode>) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if clavier.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction.y += 1.0;
    }
    if clavier.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction.y -= 1.0;
    }
    if clavier.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction.x -= 1.0;
    }
    if clavier.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction.x += 1.0;
    }
    direction.normalize_or_zero()
}

/// Dans le menu ou à la fin d'une partie : lance une partie quand la touche Entrée vient d'être
/// enfoncée.
pub fn lancer_la_partie(
    clavier: Res<ButtonInput<KeyCode>>,
    mut prochain_etat: ResMut<NextState<EtatJeu>>,
) {
    if clavier.just_pressed(KeyCode::Enter) {
        prochain_etat.set(EtatJeu::EnJeu);
    }
}

/// Au début d'une partie : fait apparaître le joueur au centre, et remet à zéro le score et les
/// minuteurs.
pub fn preparer_la_partie(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut chrono: ResMut<ChronoPartie>,
    mut minuteur: ResMut<MinuteurPieces>,
) {
    commands.spawn((Joueur, Transform::default(), DespawnOnExit(EtatJeu::EnJeu)));
    score.0 = 0;
    *chrono = ChronoPartie::default();
    *minuteur = MinuteurPieces::default();
}

/// Déplace le joueur selon le clavier, sans qu'il puisse sortir de l'arène.
pub fn deplacer_le_joueur(
    clavier: Res<ButtonInput<KeyCode>>,
    temps: Res<Time>,
    mut joueur: Single<&mut Transform, With<Joueur>>,
) {
    let deplacement = direction(&clavier) * VITESSE_JOUEUR * temps.delta_secs();
    let limite = Vec2::new(
        DEMI_LARGEUR_ARENE - TAILLE_JOUEUR / 2.0,
        DEMI_HAUTEUR_ARENE - TAILLE_JOUEUR / 2.0,
    );
    let position = (joueur.translation.truncate() + deplacement).clamp(-limite, limite);
    joueur.translation = position.extend(joueur.translation.z);
}

/// Toutes les [`INTERVALLE_PIECES`] secondes, fait apparaître une pièce à une position au
/// [`Hasard`] dans l'arène, sauf s'il y a déjà [`PIECES_MAX`] pièces.
pub fn faire_apparaitre_des_pieces(
    mut commands: Commands,
    temps: Res<Time>,
    mut minuteur: ResMut<MinuteurPieces>,
    mut hasard: ResMut<Hasard>,
    pieces: Query<(), With<Piece>>,
) {
    minuteur.0.tick(temps.delta());
    if minuteur.0.just_finished() && pieces.iter().count() < PIECES_MAX {
        let position = hasard.position_dans_l_arene();
        commands.spawn((
            Piece,
            Transform::from_translation(position.extend(0.0)),
            DespawnOnExit(EtatJeu::EnJeu),
        ));
    }
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

/// Ajoute un point au [`Score`] pour chaque pièce ramassée.
pub fn compter_les_points(
    mut pieces_ramassees: MessageReader<PieceRamassee>,
    mut score: ResMut<Score>,
) {
    score.0 += pieces_ramassees.read().count() as u32;
}

/// Fait avancer le [`ChronoPartie`], et termine la partie quand il est écoulé.
pub fn decompter_le_temps(
    temps: Res<Time>,
    mut chrono: ResMut<ChronoPartie>,
    mut prochain_etat: ResMut<NextState<EtatJeu>>,
) {
    if chrono.0.tick(temps.delta()).just_finished() {
        prochain_etat.set(EtatJeu::FinDePartie);
    }
}

/// À la fin d'une partie : met à jour le [`MeilleurScore`].
pub fn enregistrer_le_meilleur_score(score: Res<Score>, mut meilleur: ResMut<MeilleurScore>) {
    meilleur.0 = meilleur.0.max(score.0);
}

/// Le plugin du jeu : toute la logique, sans l'affichage.
pub struct JeuPlugin;

impl Plugin for JeuPlugin {
    fn build(&self, app: &mut App) {
        // Les états ont besoin du `StatesPlugin`, qui fait partie de `DefaultPlugins`. On l'ajoute
        // s'il manque, pour que le jeu fonctionne aussi dans les tests.
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }
        app.init_state::<EtatJeu>()
            // Normalement ajoutée par le `InputPlugin` : ne fait rien si elle existe déjà.
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<PieceRamassee>()
            .init_resource::<Score>()
            .init_resource::<MeilleurScore>()
            .init_resource::<ChronoPartie>()
            .init_resource::<MinuteurPieces>()
            .init_resource::<Hasard>()
            .add_systems(OnEnter(EtatJeu::EnJeu), preparer_la_partie)
            .add_systems(OnEnter(EtatJeu::FinDePartie), enregistrer_le_meilleur_score)
            .add_systems(
                Update,
                lancer_la_partie.run_if(not(in_state(EtatJeu::EnJeu))),
            )
            .add_systems(
                Update,
                (
                    deplacer_le_joueur,
                    faire_apparaitre_des_pieces,
                    ramasser_les_pieces,
                    compter_les_points,
                    decompter_le_temps,
                )
                    .chain()
                    .run_if(in_state(EtatJeu::EnJeu)),
            );
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
