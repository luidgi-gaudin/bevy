//! Des outils pour l'affichage des jeux des tutoriels, avec la fonctionnalité `jeu`.

use bevy::{asset::AssetId, prelude::*, text::Font};

/// Remplace la police par défaut de Bevy par Fira Mono complète, qui contient les accents du
/// français. À ajouter **après** les `DefaultPlugins`, qui installent la police par défaut.
///
/// La police intégrée à Bevy ne contient que les caractères ASCII, pour rester légère : sans ce
/// plugin, « Entrée » s'afficherait sans son « é ».
pub fn police_avec_accents(app: &mut App) {
    let police =
        Font::from_bytes(include_bytes!("../../assets/fonts/FiraMono-Medium.ttf").to_vec());
    app.world_mut()
        .resource_mut::<Assets<Font>>()
        .insert(AssetId::default(), police)
        .expect("la police par défaut a un identifiant fixe, toujours valide");
}
