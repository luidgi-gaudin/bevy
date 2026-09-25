//! L'arène FPS : les chapitres 10 à 17 réunis dans un match à mort contre des bots.
//!
//! ```text
//! cargo run -p bevy_tutorials --example arene_fps --features jeu --release
//! ```
//!
//! Toute la logique du jeu vient des solutions des chapitres, testées sans fenêtre : la visée à la
//! souris (chapitre 10), le mouvement à 128 ticks par seconde (11), les collisions (12), le tir,
//! l'épaulement et le recul (13 et 14), le match à mort (15), les bots (16) et la compensation de
//! latence (17). Cet exemple n'ajoute que ce que le joueur voit : la carte, le corps des bots,
//! l'arme en main, les traçantes et l'interface.
//!
//! Échap met le jeu en pause et affiche les commandes.

use bevy::{
    camera::visibility::RenderLayers,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    light::{CascadeShadowConfigBuilder, NotShadowCaster},
    math::ops,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PresentMode, PrimaryWindow},
};
use bevy_tutorials::affichage::police_avec_accents;
use bevy_tutorials::chapitre_10::solution::{
    centimetres_par_tour, rotation, viser, Joueur, Orientation, Sensibilite,
};
use bevy_tutorials::chapitre_11::solution::{
    interpoler, nouveau_joueur, plugin as plugin_du_mouvement, simuler_le_mouvement, Commandes,
    Physique, TICKS_PAR_SECONDE,
};
use bevy_tutorials::chapitre_12::solution::{
    plugin as plugin_des_collisions, resoudre_les_collisions, Obstacle,
};
use bevy_tutorials::chapitre_13::solution::{
    premier_impact, Arme, CommandesDeTir, EtatArme, Hitboxes, Impact, Tir, Touche, Zone,
    HAUTEUR_DES_YEUX, PORTEE_MAX,
};
use bevy_tutorials::chapitre_14::solution::{champ_de_vision, dispersion, Recul, Visee};
use bevy_tutorials::chapitre_15::solution::{
    plugin as plugin_du_match, Combattant, Elimination, FilDesEliminations, Match, Mort,
    PointDApparition, Vie, ELIMINATIONS_POUR_GAGNER, VIE_MAX,
};
use bevy_tutorials::chapitre_16::solution::{
    orientation_vers, penser, plugin as plugin_des_bots, Bot, Niveau,
};
use bevy_tutorials::chapitre_17::solution::{
    plugin as plugin_du_tir, position_vue, Historique, Latence, Tick,
};
use core::cmp::Reverse;
use Matiere::{Beton, Caisse, Muret, Plateforme};

/// Les matières des obstacles de la carte, qui donnent leur couleur.
#[derive(Clone, Copy)]
enum Matiere {
    Beton,
    Muret,
    Caisse,
    Plateforme,
}

/// La couleur de chaque matière, dans l'ordre de [`Matiere`].
const COULEURS: [Color; 4] = [
    Color::srgb(0.62, 0.62, 0.6),
    Color::srgb(0.36, 0.45, 0.56),
    Color::srgb(0.55, 0.4, 0.22),
    Color::srgb(0.42, 0.5, 0.4),
];

/// Un obstacle de la carte : son centre, sa taille et sa matière.
const fn boite(centre: [f32; 3], taille: [f32; 3], matiere: Matiere) -> (Vec3, Vec3, Matiere) {
    (Vec3::from_array(centre), Vec3::from_array(taille), matiere)
}

/// Les obstacles de la carte, une arène de 40 mètres sur 40 : ce sont les boîtes de collision du
/// chapitre 12.
const CARTE: [(Vec3, Vec3, Matiere); 23] = [
    // Les murs d'enceinte.
    boite([0.0, 2.0, -20.5], [42.0, 4.0, 1.0], Beton),
    boite([0.0, 2.0, 20.5], [42.0, 4.0, 1.0], Beton),
    boite([-20.5, 2.0, 0.0], [1.0, 4.0, 40.0], Beton),
    boite([20.5, 2.0, 0.0], [1.0, 4.0, 40.0], Beton),
    // Les murs qui séparent les trois couloirs, avec un passage au milieu.
    boite([-7.0, 1.5, -7.5], [1.0, 3.0, 9.0], Beton),
    boite([7.0, 1.5, -7.5], [1.0, 3.0, 9.0], Beton),
    boite([-7.0, 1.5, 7.5], [1.0, 3.0, 9.0], Beton),
    boite([7.0, 1.5, 7.5], [1.0, 3.0, 9.0], Beton),
    // La plateforme centrale : un mètre de haut, on y monte en sautant.
    boite([0.0, 0.5, 0.0], [4.0, 1.0, 4.0], Plateforme),
    // Des murets pour se couvrir. Les plus bas laissent tirer par-dessus : les yeux sont à 1,6 m.
    boite([-13.5, 0.55, 0.0], [5.0, 1.1, 0.6], Muret),
    boite([13.5, 0.55, 0.0], [5.0, 1.1, 0.6], Muret),
    boite([0.0, 1.0, -15.0], [4.0, 2.0, 1.0], Muret),
    boite([0.0, 1.0, 15.0], [4.0, 2.0, 1.0], Muret),
    // Des piliers.
    boite([-13.0, 2.0, -10.0], [1.2, 4.0, 1.2], Beton),
    boite([13.0, 2.0, -10.0], [1.2, 4.0, 1.2], Beton),
    boite([-13.0, 2.0, 10.0], [1.2, 4.0, 1.2], Beton),
    boite([13.0, 2.0, 10.0], [1.2, 4.0, 1.2], Beton),
    // Des caisses.
    boite([-15.0, 0.75, -13.0], [1.5, 1.5, 1.5], Caisse),
    boite([15.0, 0.75, 13.0], [1.5, 1.5, 1.5], Caisse),
    boite([3.5, 0.6, -9.0], [1.2, 1.2, 1.2], Caisse),
    boite([-3.5, 0.6, 9.0], [1.2, 1.2, 1.2], Caisse),
    boite([-3.5, 0.6, -5.0], [1.2, 1.2, 1.2], Caisse),
    boite([3.5, 0.6, 5.0], [1.2, 1.2, 1.2], Caisse),
];

/// Les points d'apparition, dans l'ordre de la patrouille des bots : le tour de l'arène, où rien
/// ne leur barre la route (les bots du chapitre 16 ne savent pas contourner un obstacle).
const POINTS_D_APPARITION: [Vec3; 8] = [
    Vec3::new(-17.0, 0.0, -17.0),
    Vec3::new(0.0, 0.0, -18.0),
    Vec3::new(17.0, 0.0, -17.0),
    Vec3::new(17.0, 0.0, 0.0),
    Vec3::new(17.0, 0.0, 17.0),
    Vec3::new(0.0, 0.0, 18.0),
    Vec3::new(-17.0, 0.0, 17.0),
    Vec3::new(-17.0, 0.0, 0.0),
];

/// Les noms des bots.
const NOMS_DES_BOTS: [&str; 5] = ["Alpha", "Bravo", "Charlie", "Delta", "Echo"];

/// Les niveaux des bots, choisis avec les touches 1, 2 et 3.
const DIFFICULTES: [(&str, Niveau); 3] = [
    ("facile", Niveau::FACILE),
    (
        "moyen",
        Niveau {
            temps_de_reaction: 0.4,
            vitesse_de_rotation: 180.0,
            tolerance: 2.0,
            hauteur_visee: 1.2,
        },
    ),
    ("difficile", Niveau::DIFFICILE),
];

/// Les latences simulées, en millisecondes, choisies avec la touche L.
const LATENCES: [u32; 5] = [0, 50, 100, 200, 300];

/// La couche de rendu de l'arme en main, dessinée par une deuxième caméra.
const COUCHE_DE_L_ARME: usize = 1;

/// La position de l'arme en main par rapport à la caméra, à la hanche.
const ARME_A_LA_HANCHE: Vec3 = Vec3::new(0.16, -0.15, -0.38);

/// La position de l'arme en main quand on épaule : le haut du guidon est au centre de l'écran.
const ARME_A_L_EPAULE: Vec3 = Vec3::new(0.0, -0.09, -0.3);

/// L'épaule d'un bot, au-dessus de ses pieds : son arme tourne autour de ce point.
const EPAULE_DU_BOT: Vec3 = Vec3::new(0.0, 1.35, 0.0);

/// Le centre de l'arme d'un bot, et le bout de son canon, par rapport à son épaule, quand il
/// regarde droit devant lui.
const ARME_DU_BOT: Vec3 = Vec3::new(0.3, 0.0, -0.25);
const CANON_DU_BOT: Vec3 = Vec3::new(0.3, 0.0, -0.55);

/// La durée d'affichage d'une traçante, et celle d'un impact sur un mur, en secondes.
const DUREE_DES_TRACANTES: f32 = 0.05;
const DUREE_DES_IMPACTS: f32 = 2.0;

/// Les commandes, affichées pendant la pause.
const COMMANDES: &str = "\
Souris       viser          Clic gauche  tirer
Clic droit   épauler        R            recharger
ZQSD / WASD  se déplacer    Maj          sprinter
Espace       sauter         Échap        pause
1 2 3        difficulté     L            latence simulée
- +          sensibilité";

/// La caméra du joueur, à la hauteur de ses yeux.
#[derive(Component)]
struct CameraDuJoueur;

/// L'arme que le joueur tient en main.
#[derive(Component, Default)]
struct ArmeEnMain {
    /// La secousse de l'arme juste après un tir, de 1 à 0.
    secousse: f32,
}

/// La flamme au bout du canon, visible juste après un tir.
#[derive(Component)]
struct Flamme;

/// L'arme d'un bot, qui montre où il regarde.
#[derive(Component)]
struct ArmeDeBot;

/// Le temps depuis lequel un bot en patrouille n'avance plus, en secondes.
#[derive(Component, Default)]
struct Coince(f32);

/// Les réglages choisis au clavier. Au départ : des bots faciles, sans latence.
#[derive(Resource, Default)]
struct Reglages {
    /// L'indice du niveau des bots dans [`DIFFICULTES`].
    difficulte: usize,
    /// L'indice de la latence simulée dans [`LATENCES`].
    latence: usize,
}

/// Une balle tirée, pour l'affichage : sa traçante, puis son impact.
struct Trace {
    debut: Vec3,
    fin: Vec3,
    sur_un_mur: bool,
    age: f32,
}

/// Les balles tirées récemment.
#[derive(Resource, Default)]
struct Traces(Vec<Trace>);

/// Le marqueur de touche, au centre de l'écran.
#[derive(Resource, Default)]
struct MarqueurDeTouche {
    /// Le temps d'affichage restant, en secondes.
    restant: f32,
    /// Rouge pour une élimination, blanc pour une simple touche.
    elimination: bool,
}

/// Les maillages et les matériaux des bots, partagés par tous.
#[derive(Resource)]
struct Apparence {
    /// Un pavé par hitbox : le corps des bots est exactement ce que les balles touchent.
    corps: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Transform)>,
    arme: Handle<Mesh>,
    metal: Handle<StandardMaterial>,
}

impl FromWorld for Apparence {
    fn from_world(world: &mut World) -> Self {
        let parties: Vec<(Vec3, Vec3, Zone)> = Hitboxes::default()
            .0
            .iter()
            .map(|hitbox| {
                let (min, max) = (Vec3::from(hitbox.boite.min), Vec3::from(hitbox.boite.max));
                ((min + max) / 2.0, max - min, hitbox.zone)
            })
            .collect();
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let maillages: Vec<Handle<Mesh>> = parties
            .iter()
            .map(|&(_, taille, _)| meshes.add(Cuboid::from_size(taille)))
            .collect();
        let arme = meshes.add(Cuboid::new(0.08, 0.1, 0.6));

        let mut materiaux = world.resource_mut::<Assets<StandardMaterial>>();
        let corps = parties
            .iter()
            .zip(maillages)
            .map(|(&(centre, _, zone), maillage)| {
                let couleur = match zone {
                    Zone::Tete => Color::srgb(0.95, 0.6, 0.4),
                    Zone::Torse => Color::srgb(0.75, 0.12, 0.1),
                    Zone::Jambes => Color::srgb(0.3, 0.08, 0.08),
                };
                (
                    maillage,
                    materiaux.add(couleur),
                    Transform::from_translation(centre),
                )
            })
            .collect();
        let metal = materiaux.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.12),
            metallic: 0.5,
            perceptual_roughness: 0.45,
            ..default()
        });
        Self { corps, arme, metal }
    }
}

// Les éléments de l'interface.

#[derive(Component)]
struct BrancheDuReticule(Vec2);

#[derive(Component)]
struct BrancheDuMarqueur;

#[derive(Component)]
struct FiltreDeDegats;

#[derive(Component)]
struct VoileDePause;

#[derive(Component)]
struct TexteVie;

#[derive(Component)]
struct BarreDeVie;

#[derive(Component)]
struct TexteMunitions;

#[derive(Component)]
struct TexteScores;

#[derive(Component)]
struct TexteFil;

#[derive(Component)]
struct TexteCentral;

#[derive(Component)]
struct TexteCommandes;

#[derive(Component)]
struct TexteReglages;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Arène FPS".to_string(),
                    // Sans synchronisation verticale : chaque image est affichée dès qu'elle est
                    // prête, pour le moins de latence possible entre la souris et l'écran.
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                // La souris est capturée : le curseur est caché et ne sort pas de la fenêtre.
                primary_cursor_options: Some(CursorOptions {
                    visible: false,
                    grab_mode: CursorGrabMode::Locked,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            police_avec_accents,
            // Toute la logique du jeu : les plugins des chapitres. Celui du chapitre 17 remplace
            // ceux des chapitres 13 et 14.
            (
                plugin_du_mouvement,
                plugin_des_collisions,
                plugin_du_tir,
                plugin_du_match,
                plugin_des_bots,
            ),
        ))
        .insert_resource(ClearColor(Color::srgb(0.55, 0.66, 0.8)))
        .insert_resource(GlobalAmbientLight {
            brightness: 250.0,
            ..default()
        })
        .init_resource::<Sensibilite>()
        .init_resource::<Reglages>()
        .init_resource::<Traces>()
        .init_resource::<MarqueurDeTouche>()
        .init_resource::<Apparence>()
        .add_systems(
            Startup,
            (
                installer_la_carte,
                installer_les_combattants,
                installer_l_interface,
            ),
        )
        .add_systems(
            RunFixedMainLoop,
            (
                // La souris est lue juste avant les ticks, comme le clavier (chapitre 11) : une
                // balle part dans la direction visée pendant la même image. (Le plugin du
                // chapitre 10 la lisait plus tard, dans `Update`.)
                viser
                    .run_if(souris_capturee)
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
                (afficher_avec_la_latence, placer_la_camera)
                    .after(interpoler)
                    .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                epauler_les_bots.after(penser).before(simuler_le_mouvement),
                decoincer_les_bots.after(resoudre_les_collisions),
            ),
        )
        .add_systems(
            Update,
            (
                (capturer_la_souris, mettre_en_pause).chain(),
                changer_les_reglages,
                nouvelle_partie,
                orienter_les_armes_des_bots,
                (enregistrer_les_traces, dessiner_les_traces).chain(),
                animer_l_arme,
                (detecter_les_touches, afficher_le_marqueur).chain(),
                (
                    afficher_le_reticule,
                    afficher_la_vie,
                    afficher_les_munitions,
                    afficher_les_scores,
                    afficher_le_fil,
                    afficher_le_message,
                    afficher_les_reglages,
                ),
            ),
        )
        .add_observer(habiller_les_bots)
        .add_observer(cacher_les_morts)
        .add_observer(ramener_a_la_vie)
        .run();
}

/// Crée la carte : le sol, les obstacles, les points d'apparition et le soleil.
fn installer_la_carte(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<StandardMaterial>>,
) {
    // Le sol, en y = 0, comme celui du chapitre 11.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 40.0))),
        MeshMaterial3d(materiaux.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.31, 0.33),
            perceptual_roughness: 0.95,
            ..default()
        })),
    ));

    // Chaque obstacle est dessiné par un pavé de la même taille que sa boîte de collision : ce que
    // l'on voit est exactement ce qui arrête les joueurs et les balles. Tous partagent le même
    // cube, étiré à la bonne taille, et un matériau par matière : Bevy les dessine en quelques
    // appels au GPU.
    let cube = meshes.add(Cuboid::from_size(Vec3::ONE));
    let materiaux_des_obstacles = COULEURS.map(|couleur| {
        materiaux.add(StandardMaterial {
            base_color: couleur,
            perceptual_roughness: 0.9,
            ..default()
        })
    });
    for (centre, taille, matiere) in CARTE {
        commands.spawn((
            Obstacle::new(centre, taille),
            Mesh3d(cube.clone()),
            MeshMaterial3d(materiaux_des_obstacles[matiere as usize].clone()),
            Transform::from_translation(centre).with_scale(taille),
        ));
    }

    for position in POINTS_D_APPARITION {
        commands.spawn(PointDApparition(position));
    }

    // Le soleil, qui éclaire aussi l'arme en main.
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 10.0,
            maximum_distance: 60.0,
            ..default()
        }
        .build(),
        RenderLayers::from_layers(&[0, COUCHE_DE_L_ARME]),
    ));
}

/// Crée le joueur, sa caméra et son arme en main, puis les bots.
fn installer_les_combattants(
    mut commands: Commands,
    reglages: Res<Reglages>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<StandardMaterial>>,
    apparence: Res<Apparence>,
) {
    let arme = Arme::fusil_d_assaut();
    // Tous les combattants ont le même équipement : les bots n'ont aucun avantage.
    let equipement = || {
        (
            Vie::default(),
            Hitboxes::default(),
            EtatArme::charge(&arme),
            arme.clone(),
            CommandesDeTir::default(),
            Visee::default(),
            Recul::fusil_d_assaut(),
            Historique::default(),
        )
    };

    // Le joueur.
    let depart = POINTS_D_APPARITION[0];
    commands
        .spawn((
            nouveau_joueur(depart),
            Combattant::new("Vous"),
            Latence(0),
            equipement(),
        ))
        .insert(orientation_vers(depart, Vec3::ZERO));

    // Sa caméra, et l'arme qu'il tient en main.
    let couche_de_l_arme = RenderLayers::layer(COUCHE_DE_L_ARME);
    let flamme = materiaux.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.8, 0.4),
        emissive: LinearRgba::rgb(30.0, 18.0, 6.0),
        unlit: true,
        ..default()
    });
    commands.spawn((
        CameraDuJoueur,
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: champ_de_vision(0.0),
            near: 0.05,
            ..default()
        }),
        Transform::from_translation(depart + Vec3::Y * HAUTEUR_DES_YEUX),
        children![
            // Une deuxième caméra ne dessine que l'arme en main, par-dessus la scène : l'arme ne
            // rentre jamais dans les murs, et garde la même taille quand le champ de vision se
            // resserre.
            (
                Camera3d::default(),
                Camera {
                    order: 1,
                    ..default()
                },
                Projection::from(PerspectiveProjection {
                    fov: 50.0_f32.to_radians(),
                    near: 0.01,
                    ..default()
                }),
                couche_de_l_arme.clone(),
            ),
            (
                ArmeEnMain::default(),
                Mesh3d(meshes.add(Cuboid::new(0.05, 0.08, 0.5))),
                MeshMaterial3d(apparence.metal.clone()),
                Transform::from_translation(ARME_A_LA_HANCHE),
                couche_de_l_arme.clone(),
                NotShadowCaster,
                children![
                    // Le chargeur.
                    (
                        Mesh3d(meshes.add(Cuboid::new(0.04, 0.14, 0.08))),
                        MeshMaterial3d(apparence.metal.clone()),
                        Transform::from_xyz(0.0, -0.1, -0.02),
                        couche_de_l_arme.clone(),
                        NotShadowCaster,
                    ),
                    // Le guidon, que l'on aligne sur la cible en épaulant.
                    (
                        Mesh3d(meshes.add(Cuboid::new(0.008, 0.05, 0.008))),
                        MeshMaterial3d(apparence.metal.clone()),
                        Transform::from_xyz(0.0, 0.065, -0.23),
                        couche_de_l_arme.clone(),
                        NotShadowCaster,
                    ),
                    (
                        Flamme,
                        Mesh3d(meshes.add(Sphere::new(0.035))),
                        MeshMaterial3d(flamme),
                        Transform::from_xyz(0.0, 0.0, -0.29),
                        Visibility::Hidden,
                        couche_de_l_arme,
                        NotShadowCaster,
                    ),
                ],
            ),
        ],
    ));

    // Les bots, loin du joueur.
    for (nom, &position) in NOMS_DES_BOTS.into_iter().zip(&POINTS_D_APPARITION[2..]) {
        commands.spawn((
            Bot::new(DIFFICULTES[reglages.difficulte].1),
            Coince::default(),
            Combattant::new(nom),
            orientation_vers(position, Vec3::ZERO),
            Commandes::default(),
            Physique {
                position,
                position_precedente: position,
                vitesse: Vec3::ZERO,
                au_sol: true,
            },
            Transform::from_translation(position),
            Visibility::default(),
            equipement(),
        ));
    }
}

/// Crée l'interface : les voiles, le réticule, le marqueur de touche, la vie, les munitions, les
/// scores, le fil des éliminations, les messages et les réglages.
fn installer_l_interface(mut commands: Commands) {
    let plein_ecran = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    // Un voile rouge, de plus en plus visible quand la vie baisse, et un voile sombre pour la
    // pause. Créés en premier, ils sont dessinés sous le reste de l'interface.
    commands.spawn((
        FiltreDeDegats,
        plein_ecran.clone(),
        BackgroundColor(Color::NONE),
    ));
    commands.spawn((VoileDePause, plein_ecran, BackgroundColor(Color::NONE)));

    // Le centre de l'écran : le réticule et le marqueur de touche sont placés autour de lui.
    let centre = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            ..default()
        })
        .id();
    let trait_centre = |largeur: f32, hauteur: f32| Node {
        position_type: PositionType::Absolute,
        left: px(-largeur / 2.0),
        top: px(-hauteur / 2.0),
        width: px(largeur),
        height: px(hauteur),
        ..default()
    };
    for direction in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        let (largeur, hauteur) = if direction.x == 0.0 {
            (2.0, 10.0)
        } else {
            (10.0, 2.0)
        };
        commands.spawn((
            BrancheDuReticule(direction),
            trait_centre(largeur, hauteur),
            BackgroundColor(Color::WHITE),
            UiTransform::default(),
            ChildOf(centre),
        ));
    }
    for (x, y) in [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
        commands.spawn((
            BrancheDuMarqueur,
            trait_centre(2.0, 10.0),
            BackgroundColor(Color::NONE),
            UiTransform {
                translation: Val2::px(9.0 * x, 9.0 * y),
                rotation: Rot2::degrees(-45.0 * x * y),
                ..default()
            },
            ChildOf(centre),
        ));
    }

    // Le fil des éliminations, en haut à gauche, et les scores, en haut à droite.
    let texte = |taille: f32| {
        (
            Text::default(),
            TextFont::from_font_size(taille),
            TextShadow::default(),
        )
    };
    commands.spawn((
        TexteFil,
        texte(16.0),
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            top: px(16),
            ..default()
        },
    ));
    commands.spawn((
        TexteScores,
        texte(16.0),
        Node {
            position_type: PositionType::Absolute,
            right: px(16),
            top: px(16),
            ..default()
        },
    ));

    // La vie, en bas à gauche, et les munitions, en bas à droite.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            bottom: px(32),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        },
        children![
            (TexteVie, texte(32.0)),
            (
                Node {
                    width: px(200),
                    height: px(8),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                children![(
                    BarreDeVie,
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.9, 0.95, 0.9)),
                )],
            ),
        ],
    ));
    commands.spawn((
        TexteMunitions,
        texte(32.0),
        Node {
            position_type: PositionType::Absolute,
            right: px(24),
            bottom: px(32),
            ..default()
        },
    ));

    // Les messages au centre : le titre (pause, élimination, fin du match) et les commandes.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            top: percent(20),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(32),
            ..default()
        },
        children![
            (
                TexteCentral,
                texte(26.0),
                TextLayout::justify(Justify::Center)
            ),
            (
                TexteCommandes,
                Text::new(COMMANDES),
                TextFont::from_font_size(16.0),
                TextShadow::default(),
                Visibility::Hidden,
            ),
        ],
    ));

    // Les réglages, en bas au centre.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            bottom: px(6),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(TexteReglages, texte(13.0))],
    ));
}

/// Donne un corps aux bots : un pavé par hitbox, et une arme qui montre où ils regardent.
fn habiller_les_bots(ajout: On<Add<Bot>>, mut commands: Commands, apparence: Res<Apparence>) {
    commands.entity(ajout.entity).with_children(|corps| {
        for (maillage, materiau, transform) in &apparence.corps {
            corps.spawn((
                Mesh3d(maillage.clone()),
                MeshMaterial3d(materiau.clone()),
                *transform,
            ));
        }
        corps.spawn((
            ArmeDeBot,
            Mesh3d(apparence.arme.clone()),
            MeshMaterial3d(apparence.metal.clone()),
            Transform::from_translation(EPAULE_DU_BOT + ARME_DU_BOT),
        ));
    });
}

/// Un combattant éliminé disparaît jusqu'à sa réapparition.
fn cacher_les_morts(ajout: On<Add<Mort>>, mut visibilites: Query<&mut Visibility>) {
    if let Ok(mut visibilite) = visibilites.get_mut(ajout.entity) {
        *visibilite = Visibility::Hidden;
    }
}

/// À sa réapparition, un combattant redevient visible et regarde vers le centre de l'arène. Son
/// historique est effacé : la compensation de latence ne doit pas replacer ses hitbox là où il
/// est mort.
fn ramener_a_la_vie(
    retrait: On<Remove<Mort>>,
    mut combattants: Query<(
        &Physique,
        &mut Orientation,
        &mut Historique,
        Option<&mut Visibility>,
    )>,
) {
    if let Ok((physique, mut orientation, mut historique, visibilite)) =
        combattants.get_mut(retrait.entity)
    {
        *orientation = orientation_vers(physique.position, Vec3::ZERO);
        *historique = Historique::default();
        if let Some(mut visibilite) = visibilite {
            *visibilite = Visibility::Inherited;
        }
    }
}

/// Une condition : la souris est capturée par la fenêtre.
fn souris_capturee(curseur: Single<&CursorOptions, With<PrimaryWindow>>) -> bool {
    curseur.grab_mode != CursorGrabMode::None
}

/// Échap libère la souris, un clic dans la fenêtre la capture de nouveau.
fn capturer_la_souris(
    clavier: Res<ButtonInput<KeyCode>>,
    souris: Res<ButtonInput<MouseButton>>,
    mut curseur: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if clavier.just_pressed(KeyCode::Escape) {
        curseur.grab_mode = CursorGrabMode::None;
        curseur.visible = true;
    } else if souris.just_pressed(MouseButton::Left) && curseur.grab_mode == CursorGrabMode::None {
        curseur.grab_mode = CursorGrabMode::Locked;
        curseur.visible = false;
    }
}

/// Met le jeu en pause quand la souris est libérée, ou que le match est terminé. En pause, le
/// temps virtuel s'arrête, et avec lui les ticks : plus rien ne bouge.
fn mettre_en_pause(
    curseur: Single<&CursorOptions, With<PrimaryWindow>>,
    le_match: Res<Match>,
    mut temps: ResMut<Time<Virtual>>,
) {
    let en_pause = curseur.grab_mode == CursorGrabMode::None || le_match.vainqueur.is_some();
    if en_pause && !temps.is_paused() {
        temps.pause();
    } else if !en_pause && temps.is_paused() {
        temps.unpause();
    }
}

/// Convertit une latence en millisecondes en un nombre de ticks.
fn ticks_de_latence(millisecondes: u32) -> u32 {
    (f64::from(millisecondes) * TICKS_PAR_SECONDE / 1000.0).round() as u32
}

/// Les touches 1, 2 et 3 changent le niveau des bots, L la latence simulée, - et + la sensibilité
/// de la souris.
fn changer_les_reglages(
    clavier: Res<ButtonInput<KeyCode>>,
    mut reglages: ResMut<Reglages>,
    mut sensibilite: ResMut<Sensibilite>,
    mut bots: Query<&mut Bot>,
    mut latence: Single<&mut Latence, With<Joueur>>,
) {
    for (touche, difficulte) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
    ] {
        if clavier.just_pressed(touche) {
            reglages.difficulte = difficulte;
        }
    }
    if clavier.just_pressed(KeyCode::KeyL) {
        reglages.latence = (reglages.latence + 1) % LATENCES.len();
    }
    if clavier.any_just_pressed([KeyCode::Equal, KeyCode::NumpadAdd]) {
        sensibilite.0 = (sensibilite.0 * 1.1).min(10.0);
    }
    if clavier.any_just_pressed([KeyCode::Minus, KeyCode::NumpadSubtract]) {
        sensibilite.0 = (sensibilite.0 / 1.1).max(0.1);
    }
    if reglages.is_changed() {
        for mut bot in &mut bots {
            bot.niveau = DIFFICULTES[reglages.difficulte].1;
        }
        latence.0 = ticks_de_latence(LATENCES[reglages.latence]);
    }
}

/// Entrée relance un match terminé : les scores repartent de zéro, et chaque combattant repart
/// d'un point d'apparition différent, face au centre de l'arène.
fn nouvelle_partie(
    mut commands: Commands,
    clavier: Res<ButtonInput<KeyCode>>,
    mut le_match: ResMut<Match>,
    mut fil: ResMut<FilDesEliminations>,
    mut combattants: Query<(
        Entity,
        &mut Combattant,
        &mut Vie,
        &mut Physique,
        &mut Orientation,
        &mut Historique,
        &Arme,
        &mut EtatArme,
    )>,
) {
    if le_match.vainqueur.is_none() || !clavier.just_pressed(KeyCode::Enter) {
        return;
    }
    *le_match = Match::default();
    fil.0.clear();
    for (numero, combattant) in combattants.iter_mut().enumerate() {
        let (
            entite,
            mut stats,
            mut vie,
            mut physique,
            mut orientation,
            mut historique,
            arme,
            mut etat,
        ) = combattant;
        let position = POINTS_D_APPARITION[numero % POINTS_D_APPARITION.len()];
        stats.eliminations = 0;
        stats.morts = 0;
        *vie = Vie::default();
        *physique = Physique {
            position,
            position_precedente: position,
            vitesse: Vec3::ZERO,
            au_sol: true,
        };
        *orientation = orientation_vers(position, Vec3::ZERO);
        *historique = Historique::default();
        *etat = EtatArme {
            balles_tirees: etat.balles_tirees,
            ..EtatArme::charge(arme)
        };
        commands.entity(entite).remove::<Mort>().insert((
            Hitboxes::default(),
            Commandes::default(),
            CommandesDeTir::default(),
        ));
    }
}

/// Avec de la latence simulée, les autres combattants sont affichés là où ils étaient il y a
/// `latence` ticks : c'est ce que verrait un joueur en ligne, et ce que la compensation de latence
/// du chapitre 17 reconstitue pour juger ses tirs (jusqu'à 200 ms).
fn afficher_avec_la_latence(
    tick: Res<Tick>,
    temps_fixe: Res<Time<Fixed>>,
    joueur: Single<&Latence, With<Joueur>>,
    mut autres: Query<(&Physique, &Historique, &mut Transform), Without<Joueur>>,
) {
    let latence = joueur.0;
    if latence == 0 {
        return;
    }
    let tick_vu = tick.0.saturating_sub(latence);
    let avancement = temps_fixe.overstep_fraction();
    for (physique, historique, mut transform) in &mut autres {
        let position_au_tick = |tick| {
            historique
                .position_au_tick(tick)
                .unwrap_or(physique.position)
        };
        transform.translation =
            position_au_tick(tick_vu.saturating_sub(1)).lerp(position_au_tick(tick_vu), avancement);
    }
}

/// Place la caméra aux yeux du joueur, dans la direction de son regard, avec le champ de vision
/// qui se resserre quand il épaule.
fn placer_la_camera(
    joueur: Single<(&Transform, &Orientation, &Visee), With<Joueur>>,
    camera: Single<(&mut Transform, &mut Projection), (With<CameraDuJoueur>, Without<Joueur>)>,
) {
    let (pieds, orientation, visee) = *joueur;
    let (mut transform, mut projection) = camera.into_inner();
    transform.translation = pieds.translation + Vec3::Y * HAUTEUR_DES_YEUX;
    transform.rotation = rotation(orientation);
    let champ = champ_de_vision(visee.progression);
    // Seulement s'il change : sinon, Bevy recalculerait la projection à chaque image.
    if let Projection::Perspective(perspective) = projection.bypass_change_detection()
        && perspective.fov != champ
    {
        perspective.fov = champ;
        projection.set_changed();
    }
}

/// Les bots épaulent quand ils combattent, comme les joueurs : leurs tirs sont plus précis.
fn epauler_les_bots(mut bots: Query<(&Bot, &mut Visee)>) {
    for (bot, mut visee) in &mut bots {
        visee.commande = bot.cible.is_some();
    }
}

/// Un bot qui patrouille contre un obstacle n'avance plus : au bout d'une seconde, il repart vers
/// le point de passage suivant.
fn decoincer_les_bots(
    temps: Res<Time>,
    mut bots: Query<(&mut Bot, &mut Coince, &Physique, &Commandes)>,
) {
    for (mut bot, mut coince, physique, commandes) in &mut bots {
        let vitesse = Vec2::new(physique.vitesse.x, physique.vitesse.z).length();
        if bot.cible.is_none() && commandes.avant > 0.0 && vitesse < 1.0 {
            coince.0 += temps.delta_secs();
            if coince.0 > 1.0 {
                bot.point_de_passage += 1;
                coince.0 = 0.0;
            }
        } else {
            coince.0 = 0.0;
        }
    }
}

/// L'arme d'un bot suit son regard.
fn orienter_les_armes_des_bots(
    bots: Query<&Orientation, With<Bot>>,
    mut armes: Query<(&ChildOf, &mut Transform), With<ArmeDeBot>>,
) {
    for (parent, mut transform) in &mut armes {
        if let Ok(orientation) = bots.get(parent.parent()) {
            let rotation = rotation(orientation);
            transform.translation = EPAULE_DU_BOT + rotation * ARME_DU_BOT;
            transform.rotation = rotation;
        }
    }
}

/// Enregistre une traçante pour chaque balle tirée, du canon de l'arme au point d'impact.
///
/// L'impact n'est recalculé ici que pour l'affichage : le chapitre 17 a déjà décidé qui était
/// touché. Comme lui, on place les cibles là où le tireur les voyait.
fn enregistrer_les_traces(
    mut tirs: MessageReader<Tir>,
    tick: Res<Tick>,
    tireurs: Query<(
        &Transform,
        &Orientation,
        Option<&Visee>,
        Option<&Latence>,
        Has<Joueur>,
    )>,
    personnages: Query<(Entity, &Physique, &Hitboxes, &Historique)>,
    obstacles: Query<&Obstacle>,
    mut traces: ResMut<Traces>,
) {
    let obstacles: Vec<Aabb3d> = obstacles.iter().map(|obstacle| obstacle.0).collect();
    for tir in tirs.read() {
        let (Ok(direction), Ok((transform, orientation, visee, latence, est_le_joueur))) =
            (Dir3::new(tir.direction), tireurs.get(tir.tireur))
        else {
            continue;
        };
        let latence = latence.map_or(0, |latence| latence.0);
        let cibles = personnages
            .iter()
            .map(|(entite, physique, hitboxes, historique)| {
                let position = position_vue(historique, physique.position, tick.0, latence);
                (entite, position, hitboxes)
            });
        let impact = premier_impact(tir.origine, direction, &obstacles, cibles, tir.tireur);
        let rotation = rotation(orientation);
        let debut = if est_le_joueur {
            let progression = visee.map_or(0.0, |visee| visee.progression);
            let arme = ARME_A_LA_HANCHE.lerp(ARME_A_L_EPAULE, progression);
            tir.origine + rotation * (arme + Vec3::NEG_Z * 0.3)
        } else {
            transform.translation + EPAULE_DU_BOT + rotation * CANON_DU_BOT
        };
        traces.0.push(Trace {
            debut,
            fin: tir.origine + direction * impact.map_or(PORTEE_MAX, |impact| impact.distance()),
            sur_un_mur: matches!(impact, Some(Impact::Obstacle { .. })),
            age: 0.0,
        });
    }
}

/// Dessine les traçantes, très brièvement, et les impacts sur les murs, qui s'effacent peu à peu.
fn dessiner_les_traces(mut gizmos: Gizmos, temps: Res<Time>, mut traces: ResMut<Traces>) {
    let dt = temps.delta_secs();
    traces.0.retain_mut(|trace| {
        trace.age += dt;
        trace.age < DUREE_DES_IMPACTS
    });
    for trace in &traces.0 {
        if trace.age < DUREE_DES_TRACANTES {
            let opacite = 1.0 - trace.age / DUREE_DES_TRACANTES;
            gizmos.line(
                trace.debut,
                trace.fin,
                Color::srgb(1.0, 0.85, 0.45).with_alpha(opacite),
            );
        }
        if trace.sur_un_mur {
            let opacite = 1.0 - trace.age / DUREE_DES_IMPACTS;
            gizmos.sphere(
                Isometry3d::from_translation(trace.fin),
                0.04,
                Color::srgb(0.1, 0.1, 0.1).with_alpha(opacite),
            );
        }
    }
}

/// L'arme en main passe de la hanche à l'épaule, recule et crache une flamme à chaque tir.
fn animer_l_arme(
    temps: Res<Time>,
    mut tirs: MessageReader<Tir>,
    joueur: Single<(Entity, &Visee, Has<Mort>), With<Joueur>>,
    arme: Single<(&mut ArmeEnMain, &mut Transform, &mut Visibility)>,
    mut flamme: Single<&mut Visibility, (With<Flamme>, Without<ArmeEnMain>)>,
) {
    let (joueur, visee, mort) = *joueur;
    let (mut arme, mut transform, mut visibilite) = arme.into_inner();
    if tirs.read().filter(|tir| tir.tireur == joueur).count() > 0 {
        arme.secousse = 1.0;
    }
    arme.secousse = (arme.secousse - 15.0 * temps.delta_secs()).max(0.0);
    transform.translation = ARME_A_LA_HANCHE.lerp(ARME_A_L_EPAULE, visee.progression)
        + Vec3::new(0.0, 0.01, 0.04) * arme.secousse;
    visibilite.set_if_neq(if mort {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    flamme.set_if_neq(if arme.secousse > 0.6 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}

/// Détecte les balles du joueur qui touchent, et ses éliminations.
fn detecter_les_touches(
    mut touches: MessageReader<Touche>,
    mut eliminations: MessageReader<Elimination>,
    joueur: Single<Entity, With<Joueur>>,
    mut marqueur: ResMut<MarqueurDeTouche>,
) {
    if touches
        .read()
        .filter(|touche| touche.tireur == *joueur)
        .count()
        > 0
    {
        *marqueur = MarqueurDeTouche {
            restant: 0.15,
            elimination: false,
        };
    }
    if eliminations
        .read()
        .filter(|elimination| elimination.tueur == *joueur)
        .count()
        > 0
    {
        *marqueur = MarqueurDeTouche {
            restant: 0.4,
            elimination: true,
        };
    }
}

/// Le marqueur de touche : une croix blanche quand une balle touche, rouge pour une élimination.
fn afficher_le_marqueur(
    temps: Res<Time>,
    mut marqueur: ResMut<MarqueurDeTouche>,
    mut branches: Query<&mut BackgroundColor, With<BrancheDuMarqueur>>,
) {
    marqueur.restant = (marqueur.restant - temps.delta_secs()).max(0.0);
    let couleur = if marqueur.restant == 0.0 {
        Color::NONE
    } else if marqueur.elimination {
        Color::srgb(1.0, 0.2, 0.15)
    } else {
        Color::WHITE
    };
    for mut fond in &mut branches {
        fond.set_if_neq(BackgroundColor(couleur));
    }
}

/// Le réticule montre la dispersion réelle des balles (chapitre 14) : il s'écarte quand on bouge
/// ou qu'on saute, se resserre à l'arrêt, et disparaît quand on épaule.
fn afficher_le_reticule(
    fenetre: Single<&Window, With<PrimaryWindow>>,
    joueur: Single<(&Physique, &Visee, Has<Mort>), With<Joueur>>,
    mut branches: Query<(&BrancheDuReticule, &mut UiTransform, &mut BackgroundColor)>,
) {
    let (physique, visee, mort) = *joueur;
    // L'angle de dispersion, converti en pixels selon le champ de vision vertical.
    let angle = dispersion(visee.progression, physique).to_radians();
    let demi_champ = champ_de_vision(visee.progression) / 2.0;
    let ecart = ops::tan(angle) / ops::tan(demi_champ) * fenetre.height() / 2.0 + 5.0;
    let opacite = if mort { 0.0 } else { 1.0 - visee.progression };
    for (branche, mut transform, mut fond) in &mut branches {
        transform.set_if_neq(UiTransform::from_translation(Val2::px(
            branche.0.x * ecart,
            -branche.0.y * ecart,
        )));
        fond.set_if_neq(BackgroundColor(Color::WHITE.with_alpha(0.9 * opacite)));
    }
}

/// La vie, et un voile rouge de plus en plus visible quand elle baisse.
fn afficher_la_vie(
    joueur: Single<&Vie, With<Joueur>>,
    mut texte: Single<&mut Text, With<TexteVie>>,
    mut barre: Single<&mut Node, With<BarreDeVie>>,
    mut filtre: Single<&mut BackgroundColor, With<FiltreDeDegats>>,
) {
    let part = (joueur.points / VIE_MAX).clamp(0.0, 1.0);
    texte.set_if_neq(Text::new(format!("{:.0}", joueur.points.max(0.0).ceil())));
    let largeur = percent(100.0 * part);
    if barre.width != largeur {
        barre.width = largeur;
    }
    filtre.set_if_neq(BackgroundColor(Color::srgba(
        0.6,
        0.0,
        0.0,
        0.45 * (1.0 - part),
    )));
}

/// Les munitions : les balles dans le chargeur, ou le rechargement en cours.
fn afficher_les_munitions(
    joueur: Single<(&Arme, &EtatArme), With<Joueur>>,
    mut texte: Single<&mut Text, With<TexteMunitions>>,
) {
    let (arme, etat) = *joueur;
    let contenu = if etat.rechargement.is_some() {
        "Rechargement".to_string()
    } else {
        format!("{} / {}", etat.munitions, arme.taille_chargeur)
    };
    texte.set_if_neq(Text::new(contenu));
}

/// Le classement, du meilleur au moins bon.
fn afficher_les_scores(
    joueur: Single<Entity, With<Joueur>>,
    combattants: Query<(Entity, &Combattant)>,
    mut texte: Single<&mut Text, With<TexteScores>>,
) {
    let mut classement: Vec<_> = combattants.iter().collect();
    classement.sort_by_key(|(_, combattant)| (Reverse(combattant.eliminations), combattant.morts));
    let mut contenu = format!(
        "Premier à {ELIMINATIONS_POUR_GAGNER}\n\n{:<10}{:>6}{:>7}",
        "", "Élim.", "Morts"
    );
    for (entite, combattant) in classement {
        let marque = if entite == *joueur { '>' } else { ' ' };
        contenu += &format!(
            "\n{marque} {:<8}{:>6}{:>7}",
            combattant.nom, combattant.eliminations, combattant.morts
        );
    }
    texte.set_if_neq(Text::new(contenu));
}

/// Le fil des dernières éliminations.
fn afficher_le_fil(
    fil: Res<FilDesEliminations>,
    combattants: Query<&Combattant>,
    mut texte: Single<&mut Text, With<TexteFil>>,
) {
    let nom = |entite| {
        combattants
            .get(entite)
            .map_or_else(|_| "?".to_string(), |combattant| combattant.nom.clone())
    };
    let lignes: Vec<String> = fil
        .0
        .iter()
        .map(|elimination| {
            let dans_la_tete = if elimination.zone == Zone::Tete {
                "  (tête)"
            } else {
                ""
            };
            format!(
                "{} » {}{dans_la_tete}",
                nom(elimination.tueur),
                nom(elimination.victime)
            )
        })
        .collect();
    texte.set_if_neq(Text::new(lignes.join("\n")));
}

/// Les messages au centre de l'écran : la pause et les commandes, l'élimination du joueur, ou la
/// fin du match.
fn afficher_le_message(
    curseur: Single<&CursorOptions, With<PrimaryWindow>>,
    le_match: Res<Match>,
    fil: Res<FilDesEliminations>,
    joueur: Single<(Entity, Option<&Mort>), With<Joueur>>,
    combattants: Query<&Combattant>,
    mut texte: Single<&mut Text, With<TexteCentral>>,
    mut commandes: Single<&mut Visibility, With<TexteCommandes>>,
    mut voile: Single<&mut BackgroundColor, With<VoileDePause>>,
) {
    let (joueur, mort) = *joueur;
    let nom = |entite| {
        combattants
            .get(entite)
            .map_or_else(|_| "?".to_string(), |combattant| combattant.nom.clone())
    };
    let en_pause = curseur.grab_mode == CursorGrabMode::None;
    let contenu = if let Some(vainqueur) = le_match.vainqueur {
        let titre = if vainqueur == joueur {
            "VICTOIRE !".to_string()
        } else {
            format!("{} remporte le match", nom(vainqueur))
        };
        format!("{titre}\n\nEntrée : rejouer")
    } else if en_pause {
        "PAUSE\n\nCliquez pour reprendre".to_string()
    } else if let Some(mort) = mort {
        let tueur = fil
            .0
            .iter()
            .rev()
            .find(|elimination| elimination.victime == joueur)
            .map(|elimination| nom(elimination.tueur))
            .unwrap_or_default();
        format!(
            "Éliminé par {tueur}\n\nRéapparition dans {:.0} s",
            mort.reapparition.max(0.0).ceil()
        )
    } else {
        String::new()
    };
    texte.set_if_neq(Text::new(contenu));
    commandes.set_if_neq(if en_pause {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    let assombri = en_pause || le_match.vainqueur.is_some();
    voile.set_if_neq(BackgroundColor(Color::srgba(
        0.0,
        0.0,
        0.0,
        if assombri { 0.6 } else { 0.0 },
    )));
}

/// Les réglages, et le nombre d'images par seconde.
fn afficher_les_reglages(
    reglages: Res<Reglages>,
    sensibilite: Res<Sensibilite>,
    diagnostics: Res<DiagnosticsStore>,
    mut texte: Single<&mut Text, With<TexteReglages>>,
) {
    let images_par_seconde = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diagnostic| diagnostic.smoothed())
        .unwrap_or_default();
    texte.set_if_neq(Text::new(format!(
        "Bots : {} [1 2 3]    Latence simulée : {} ms [L]    \
         Sensibilité : {:.2}, soit {:.0} cm par tour à 800 DPI [- +]    {:.0} images/s",
        DIFFICULTES[reglages.difficulte].0,
        LATENCES[reglages.latence],
        sensibilite.0,
        centimetres_par_tour(sensibilite.0, 800.0),
        images_par_seconde,
    )));
}
