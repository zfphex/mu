#![allow(static_mut_refs)]
//! The physical database is a file on disk that stores song information.
//! This information includes the artist, album, title, disc number, track number, path and replay gain.
//!
//! The virtual database stores key value pairs.
//! It is used for quering artists, albums and songs.
//!
//! `Index` is a wrapper over a `Vec<T>` plus an index. Kind of like a circular buffer but the data is usually constant.
//! It's useful for moving up and down the selection of a UI element.
use std::{
    borrow::Cow,
    env,
    error::Error,
    fs::{self},
    mem::MaybeUninit,
    path::{Path, PathBuf},
    sync::Once,
};

pub use crate::{
    db::{Album, Artist, Song},
    playlist::Playlist,
};
pub use index::*;

pub mod db;
pub mod index;
pub mod log;
pub mod playlist;
pub mod settings;
pub mod strsim;
pub mod vdb;

///Escape potentially problematic strings.
pub fn escape(input: &'_ str) -> Cow<'_, str> {
    if input.contains(['\n', '\t']) {
        Cow::Owned(input.replace('\n', "").replace('\t', "    "))
    } else {
        Cow::Borrowed(input)
    }
}

//TODO: I'm not sure why I wrote this three years ago, but this code HAS TO GO.
static mut MU: MaybeUninit<PathBuf> = MaybeUninit::uninit();
static mut SETTINGS: MaybeUninit<PathBuf> = MaybeUninit::uninit();
static mut DATABASE: MaybeUninit<PathBuf> = MaybeUninit::uninit();
static mut ONCE: Once = Once::new();

pub fn user_profile_directory() -> Option<String> {
    env::var("USERPROFILE").ok()
}

#[inline(always)]
fn once() {
    unsafe {
        ONCE.call_once(|| {
            let mu = if cfg!(windows) {
                PathBuf::from(&env::var("APPDATA").unwrap())
            } else {
                PathBuf::from(&env::var("HOME").unwrap()).join(".config")
            }
            .join("mu");

            if !mu.exists() {
                fs::create_dir_all(&mu).unwrap();
            }

            let settings = mu.join("settings.db");

            //Backwards compatibility for older versions of mu
            let old_db = mu.join("mu_new.db");
            let db = mu.join("mu.db");

            if old_db.exists() {
                fs::rename(old_db, &db).unwrap();
            }

            MU = MaybeUninit::new(mu);
            SETTINGS = MaybeUninit::new(settings);
            DATABASE = MaybeUninit::new(db);
        });
    }
}

pub fn mu_path() -> &'static Path {
    once();
    unsafe { MU.assume_init_ref() }
}

pub fn settings_path() -> &'static Path {
    once();
    unsafe { SETTINGS.assume_init_ref() }
}

pub fn database_path() -> &'static Path {
    once();
    unsafe { DATABASE.assume_init_ref() }
}

trait Serialize {
    fn serialize(&self) -> String;
}

trait Deserialize
where
    Self: Sized,
{
    type Error;

    fn deserialize(s: &str) -> Result<Self, Self::Error>;
}
