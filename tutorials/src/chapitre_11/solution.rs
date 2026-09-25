//! Solution du chapitre 11 : se déplacer à tick fixe, comme sur un serveur de FPS compétitif.

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
pub fn axes_au_sol(lacet: f32) -> (Vec3, Vec3) {
    let (sinus, cosinus) = ops::sin_cos(lacet);
    (
        Vec3::new(-sinus, 0.0, -cosinus),
        Vec3::new(cosinus, 0.0, -sinus),
    )
}

/// Freine la vitesse horizontale selon la [`FRICTION`] du sol, pendant `dt` secondes. La vitesse
/// verticale ne change pas.
pub fn frotter(vitesse: Vec3, dt: f32) -> Vec3 {
    let horizontale = Vec3::new(vitesse.x, 0.0, vitesse.z);
    let norme = horizontale.length();
    if norme < 1e-4 {
        return Vec3::new(0.0, vitesse.y, 0.0);
    }
    let perte = norme.max(VITESSE_D_ARRET) * FRICTION * dt;
    let nouvelle_norme = (norme - perte).max(0.0);
    horizontale * (nouvelle_norme / norme) + Vec3::Y * vitesse.y
}

/// Accélère `vitesse` dans la `direction` (de longueur 1), sans dépasser `vitesse_voulue` dans
/// cette direction, comme dans Quake.
pub fn accelerer(
    vitesse: Vec3,
    direction: Vec3,
    vitesse_voulue: f32,
    acceleration: f32,
    dt: f32,
) -> Vec3 {
    let vitesse_actuelle = vitesse.dot(direction);
    let manque = vitesse_voulue - vitesse_actuelle;
    if manque <= 0.0 {
        return vitesse;
    }
    let ajout = (acceleration * vitesse_voulue * dt).min(manque);
    vitesse + direction * ajout
}

/// Calcule un tick de mouvement de `dt` secondes : renvoie la nouvelle physique du joueur.
///
/// Le sol est le plan horizontal `y = 0`.
pub fn tick_de_mouvement(
    physique: &Physique,
    commandes: &Commandes,
    lacet: f32,
    dt: f32,
) -> Physique {
    let mut suivant = *physique;
    suivant.position_precedente = physique.position;

    let (avant, droite) = axes_au_sol(lacet);
    let souhait = avant * commandes.avant + droite * commandes.droite;
    let direction = souhait.normalize_or_zero();
    let vitesse_max = if commandes.sprint && commandes.avant > 0.0 {
        VITESSE_SPRINT
    } else {
        VITESSE_MARCHE
    };
    let vitesse_voulue = vitesse_max * souhait.length().min(1.0);

    if physique.au_sol {
        suivant.vitesse = frotter(suivant.vitesse, dt);
        suivant.vitesse = accelerer(
            suivant.vitesse,
            direction,
            vitesse_voulue,
            ACCELERATION_SOL,
            dt,
        );
        if commandes.saut {
            suivant.vitesse.y = VITESSE_DE_SAUT;
        }
    } else {
        suivant.vitesse = accelerer(
            suivant.vitesse,
            direction,
            vitesse_voulue,
            ACCELERATION_AIR,
            dt,
        );
    }

    // La gravité s'applique toujours : au sol, le sol la compense.
    suivant.vitesse.y -= GRAVITE * dt;
    suivant.position += suivant.vitesse * dt;

    if suivant.position.y <= 0.0 {
        suivant.position.y = 0.0;
        suivant.vitesse.y = suivant.vitesse.y.max(0.0);
        suivant.au_sol = true;
    } else {
        suivant.au_sol = false;
    }
    suivant
}

/// Lit le clavier et met à jour les [`Commandes`] du joueur.
///
/// Un saut demandé reste en attente jusqu'au prochain tick : sans cela, un appui très bref,
/// pendant une image où aucun tick n'est simulé, serait perdu.
pub fn lire_le_clavier(
    clavier: Res<ButtonInput<KeyCode>>,
    mut joueurs: Query<&mut Commandes, With<Joueur>>,
) {
    let axe = |plus: [KeyCode; 2], moins: [KeyCode; 2]| {
        f32::from(u8::from(clavier.any_pressed(plus)))
            - f32::from(u8::from(clavier.any_pressed(moins)))
    };
    for mut commandes in &mut joueurs {
        commandes.avant = axe(
            [KeyCode::KeyW, KeyCode::ArrowUp],
            [KeyCode::KeyS, KeyCode::ArrowDown],
        );
        commandes.droite = axe(
            [KeyCode::KeyD, KeyCode::ArrowRight],
            [KeyCode::KeyA, KeyCode::ArrowLeft],
        );
        commandes.sprint = clavier.pressed(KeyCode::ShiftLeft);
        if clavier.just_pressed(KeyCode::Space) {
            commandes.saut = true;
        }
    }
}

/// Simule un tick de mouvement pour chaque joueur, et consomme les sauts demandés.
///
/// S'exécute dans `FixedUpdate`, où `Res<Time>` donne la durée d'un tick.
pub fn simuler_le_mouvement(
    temps: Res<Time>,
    mut joueurs: Query<(&mut Physique, &mut Commandes, &Orientation)>,
) {
    let dt = temps.delta_secs();
    for (mut physique, mut commandes, orientation) in &mut joueurs {
        *physique = tick_de_mouvement(&physique, &commandes, orientation.lacet, dt);
        commandes.saut = false;
    }
}

/// Place le `Transform` de chaque joueur entre ses positions des deux derniers ticks, selon le
/// temps écoulé depuis le dernier tick : l'affichage reste fluide, même à 240 images par seconde.
pub fn interpoler(temps_fixe: Res<Time<Fixed>>, mut joueurs: Query<(&Physique, &mut Transform)>) {
    let avancement = temps_fixe.overstep_fraction();
    for (physique, mut transform) in &mut joueurs {
        transform.translation = physique
            .position_precedente
            .lerp(physique.position, avancement);
    }
}

/// Le plugin du chapitre : 128 ticks par seconde, le clavier lu juste avant les ticks, et
/// l'interpolation juste après.
pub fn plugin(app: &mut App) {
    app.insert_resource(Time::<Fixed>::from_hz(TICKS_PAR_SECONDE))
        .add_systems(
            RunFixedMainLoop,
            (
                lire_le_clavier.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
                interpoler.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            ),
        )
        .add_systems(FixedUpdate, simuler_le_mouvement);
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
