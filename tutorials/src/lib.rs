//! Des tutoriels progressifs pour apprendre Bevy, avec des tests prêts à l'emploi.
//!
//! Chaque chapitre est un dossier de `src/` qui contient :
//! - `README.md` : la leçon,
//! - `exercice.rs` : le code à compléter, là où se trouvent les `todo!()`,
//! - `tests.rs` : les tests, qui vérifient votre exercice,
//! - `solution.rs` : une solution, à consulter en cas de blocage.
//!
//! Les tests sont lancés sur les solutions avec `cargo test -p bevy_tutorials`, et sur vos
//! exercices avec `cargo test -p bevy_tutorials --test exercices`. Voir le `README.md` des
//! tutoriels pour commencer.

// Permet aux tests d'utiliser `bevy_tutorials::outils` aussi bien depuis ce crate que depuis les
// exercices.
extern crate self as bevy_tutorials;

pub mod outils;

pub mod chapitre_01 {
    //! Premiers pas : l'application, les systèmes et les ressources.
    pub mod solution;
}

pub mod chapitre_02 {
    //! Les entités, les composants et les requêtes.
    pub mod solution;
}

pub mod chapitre_03 {
    //! Les commandes : créer, modifier et détruire des entités.
    pub mod solution;
}

pub mod chapitre_04 {
    //! Le temps : les minuteurs et les déplacements réguliers.
    pub mod solution;
}

pub mod chapitre_05 {
    //! Les entrées du clavier.
    pub mod solution;
}

pub mod chapitre_06 {
    //! Les messages et les observateurs.
    pub mod solution;
}

pub mod chapitre_07 {
    //! Les états du jeu : menu, partie, fin de partie.
    pub mod solution;
}

pub mod chapitre_08 {
    //! Organiser le jeu : plugins et ordre des systèmes.
    pub mod solution;
}

pub mod chapitre_09 {
    //! Le projet final : « Chasseur de pièces ».
    pub mod solution;
}
