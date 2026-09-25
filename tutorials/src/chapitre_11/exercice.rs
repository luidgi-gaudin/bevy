//! Exercice du chapitre 11 : se déplacer à tick fixe, comme sur un serveur de FPS compétitif.
//!
//! Remplacez chaque `todo!()` par votre code, puis lancez les tests :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices chapitre_11
//! ```
//!
//! Ce chapitre utilise `Joueur` et `Orientation` de la solution du chapitre 10.

use bevy::{math::ops, prelude::*};
use bevy_tutorials::chapitre_10::solution::{Joueur, Orientation};

/// Le nombre de ticks de simulation par seconde, comme sur les serveurs compétitifs de Valorant.
pub const TICKS_PAR_SECONDE: f64 = 128.0;

/// La vitesse de marche, en mètres par seconde.
pub const VITESSE_MARCHE: f32 = 5.0;

/// La vitesse de sprint, en mètres par seconde.
pub const VITESSE_SPRINT: f32 = 7.5;

/// L'accélération au sol : plus elle est grande, plus le joueur atteint vite sa vitesse.
pub const ACCELERATION_SOL: f32 = 10.0;

/// L'accélération en l'air : faible, pour garder un peu de contrôle pendant un saut.
pub const ACCELERATION_AIR: f32 = 1.0;

/// La friction du sol : plus elle est grande, plus le joueur s'arrête vite.
pub const FRICTION: f32 = 6.0;

/// En dessous de cette vitesse, en mètres par seconde, la friction freine comme à cette vitesse,
/// pour que le joueur s'arrête net au lieu de ralentir de plus en plus doucement.
pub const VITESSE_D_ARRET: f32 = 2.0;

/// La gravité, en mètres par seconde au carré : plus forte que sur Terre, pour des sauts
/// nerveux.
pub const GRAVITE: f32 = 20.0;

/// La vitesse verticale au début d'un saut, en mètres par seconde.
pub const VITESSE_DE_SAUT: f32 = 7.0;

/// Les commandes du joueur : lues à chaque image, utilisées à chaque tick.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Commandes {
    /// Avancer (1), reculer (-1), ou entre les deux avec une manette.
    pub avant: f32,
    /// Aller à droite (1), à gauche (-1), ou entre les deux avec une manette.
    pub droite: f32,
    /// Sprinter : seulement en avançant.
    pub sprint: bool,
    /// Un saut demandé, en attente du prochain tick.
    pub saut: bool,
}

/// L'état physique d'un joueur, simulé à chaque tick.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Physique {
    /// La position des pieds à la fin du dernier tick.
    pub position: Vec3,
    /// La position des pieds à la fin de l'avant-dernier tick, pour l'interpolation.
    pub position_precedente: Vec3,
    /// La vitesse, en mètres par seconde.
    pub vitesse: Vec3,
    /// Le joueur touche-t-il le sol ?
    pub au_sol: bool,
}

/// Renvoie les composants d'un joueur placé en `position`, les pieds au sol.
pub fn nouveau_joueur(position: Vec3) -> impl Bundle {
    (
        Joueur,
        Orientation::default(),
        Commandes::default(),
        Physique {
            position,
            position_precedente: position,
            vitesse: Vec3::ZERO,
            au_sol: position.y <= 0.0,
        },
        Transform::from_translation(position),
    )
}

/// Renvoie les directions « avant » et « droite », à l'horizontale, pour un lacet donné.
///
/// Indices : avec un lacet nul, l'avant est `-Z` et la droite `+X` ; `ops::sin_cos(angle)`
/// renvoie le sinus et le cosinus (utilisez les fonctions de `ops` plutôt que `f32::sin` : elles
/// donnent exactement le même résultat sur tous les ordinateurs).
pub fn axes_au_sol(lacet: f32) -> (Vec3, Vec3) {
    todo!()
}

/// Freine la vitesse horizontale selon la [`FRICTION`] du sol, pendant `dt` secondes. La vitesse
/// verticale ne change pas.
///
/// La vitesse horizontale perd `max(vitesse, VITESSE_D_ARRET) × FRICTION × dt`, sans devenir
/// négative, et garde sa direction.
pub fn frotter(vitesse: Vec3, dt: f32) -> Vec3 {
    todo!()
}

/// Accélère `vitesse` dans la `direction` (de longueur 1), sans dépasser `vitesse_voulue` dans
/// cette direction, comme dans Quake.
///
/// La vitesse dans la direction est `vitesse.dot(direction)`. S'il manque de la vitesse, on ajoute
/// dans la direction `acceleration × vitesse_voulue × dt`, sans dépasser ce qui manque.
pub fn accelerer(
    vitesse: Vec3,
    direction: Vec3,
    vitesse_voulue: f32,
    acceleration: f32,
    dt: f32,
) -> Vec3 {
    todo!()
}

/// Calcule un tick de mouvement de `dt` secondes : renvoie la nouvelle physique du joueur.
///
/// 1. La position actuelle devient la position précédente.
/// 2. La direction voulue combine l'avant et la droite selon les commandes ; la vitesse voulue
///    est celle du sprint (en avançant) ou de la marche.
/// 3. Au sol : friction, accélération au sol, et saut si demandé (la vitesse verticale devient
///    [`VITESSE_DE_SAUT`]). En l'air : seulement l'accélération en l'air.
/// 4. La gravité, puis le déplacement selon la vitesse.
/// 5. Le sol est le plan `y = 0` : en dessous, le joueur est replacé au sol, sans vitesse vers le
///    bas, et `au_sol` devient vrai ; sinon, il est en l'air.
pub fn tick_de_mouvement(
    physique: &Physique,
    commandes: &Commandes,
    lacet: f32,
    dt: f32,
) -> Physique {
    todo!()
}

/// Lit le clavier (flèches ou WASD/ZQSD, Maj gauche pour sprinter, Espace pour sauter) et met à
/// jour les [`Commandes`] du joueur.
///
/// Un saut demandé doit rester en attente jusqu'au prochain tick, même si la touche est relâchée
/// entre-temps.
pub fn lire_le_clavier(clavier: Res<ButtonInput<KeyCode>>) {
    todo!()
}

/// Simule un tick de mouvement pour chaque joueur, et consomme les sauts demandés.
///
/// S'exécute dans `FixedUpdate`, où `Res<Time>` donne la durée d'un tick.
pub fn simuler_le_mouvement(temps: Res<Time>) {
    todo!()
}

/// Place le `Transform` de chaque joueur entre ses positions des deux derniers ticks, selon le
/// temps écoulé depuis le dernier tick.
///
/// Indice : `Res<Time<Fixed>>` donne `overstep_fraction()`, entre 0 (on vient de simuler un tick)
/// et 1 (le prochain tick est imminent), et `a.lerp(b, t)` interpole entre deux positions.
pub fn interpoler() {
    todo!()
}

/// Le plugin du chapitre : la ressource `Time::<Fixed>::from_hz(TICKS_PAR_SECONDE)`, le clavier
/// lu dans `RunFixedMainLoop`, avant les ticks (`RunFixedMainLoopSystems::BeforeFixedMainLoop`),
/// l'interpolation après (`RunFixedMainLoopSystems::AfterFixedMainLoop`), et la simulation dans
/// `FixedUpdate`.
pub fn plugin(app: &mut App) {
    todo!()
}

// Les tests du chapitre, les mêmes que pour la solution. Ne les modifiez pas !
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
