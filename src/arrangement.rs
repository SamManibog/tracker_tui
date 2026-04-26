use std::{cell::RefCell, collections::HashMap, sync::Mutex};

use crate::Phrase;

/// the id of an instrument
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrumentId(pub u32);

impl From<u32> for InstrumentId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// the id of a phrase
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhraseId(pub u32);

impl From<u32> for PhraseId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// a track on an arrangement
#[derive(Debug)]
pub struct Track {
    instrument: InstrumentId,
    phrase_id: PhraseId,
    slots: Vec<TrackSlot>,
}

/// a slot on a track, occupied by a phrase
#[derive(Debug)]
pub struct TrackSlot {
    start: u32,
    transpose_cents: i32,
    phrase: PhraseId
}

/// data relating to arrangement of phrases in tracks
#[derive(Debug)]
pub struct Arrangement {
    wholes_per_second: f64,
    phrases: HashMap<PhraseId, Box<Phrase>>,
    tracks: Vec<Mutex<Track>>,
}

impl Default for Arrangement {
    fn default() -> Self {
        Self {
            wholes_per_second: 24.0,
            phrases: Default::default(),
            tracks: Default::default(),
        } 
    }
}

impl Arrangement {
    pub fn get_phrase(&self, id: PhraseId) -> Option<&Phrase> {
        self.phrases.get(&id).map(|item| item.as_ref())
    }

    pub fn get_phrase_mut(&mut self, id: PhraseId) -> Option<&mut Phrase> {
        self.phrases.get_mut(&id).map(|item| item.as_mut())
    }

    pub fn set_phrase(&mut self, id: PhraseId, phrase: Box<Phrase>) {
        self.phrases.insert(id, phrase);
    }
}
