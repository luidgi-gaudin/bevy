//! Lance les tests des chapitres sur vos exercices :
//!
//! ```text
//! cargo test -p bevy_tutorials --test exercices               # tous les chapitres
//! cargo test -p bevy_tutorials --test exercices chapitre_01   # un seul chapitre
//! ```
//!
//! Tant qu'un exercice contient des `todo!()`, ses tests échouent avec le message
//! `not yet implemented` : c'est normal, c'est à vous de jouer !

// Les `todo!()` des exercices laissent des variables inutilisées : on masque ces avertissements
// pour que les erreurs importantes restent visibles.
#![allow(unused_variables, unused_mut, unused_imports, dead_code)]

#[path = "../src/chapitre_01/exercice.rs"]
mod chapitre_01;

#[path = "../src/chapitre_02/exercice.rs"]
mod chapitre_02;

#[path = "../src/chapitre_03/exercice.rs"]
mod chapitre_03;

#[path = "../src/chapitre_04/exercice.rs"]
mod chapitre_04;

#[path = "../src/chapitre_05/exercice.rs"]
mod chapitre_05;

#[path = "../src/chapitre_06/exercice.rs"]
mod chapitre_06;

#[path = "../src/chapitre_07/exercice.rs"]
mod chapitre_07;

#[path = "../src/chapitre_08/exercice.rs"]
mod chapitre_08;

#[path = "../src/chapitre_09/exercice.rs"]
mod chapitre_09;

#[path = "../src/chapitre_10/exercice.rs"]
mod chapitre_10;

#[path = "../src/chapitre_11/exercice.rs"]
mod chapitre_11;

#[path = "../src/chapitre_12/exercice.rs"]
mod chapitre_12;

#[path = "../src/chapitre_13/exercice.rs"]
mod chapitre_13;

#[path = "../src/chapitre_14/exercice.rs"]
mod chapitre_14;

#[path = "../src/chapitre_15/exercice.rs"]
mod chapitre_15;

#[path = "../src/chapitre_16/exercice.rs"]
mod chapitre_16;

#[path = "../src/chapitre_17/exercice.rs"]
mod chapitre_17;
