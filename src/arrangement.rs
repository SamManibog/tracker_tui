use std::{cell::RefCell, collections::HashMap, sync::Mutex};

use crate::Phrase;

/// the id of an instrument
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrumentId(u32);

/// the id of a phrase
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhraseId(u32);

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
    phrases: HashMap<PhraseId, Phrase>,
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
        self.phrases.get(&id)
    }
}
