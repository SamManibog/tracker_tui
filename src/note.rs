use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Note(i32);

impl Note {
    /// creates a new note given the difference in semitones from a4
    pub const fn from_semitone_difference(difference: i32) -> Self {
        Self(difference * 100)
    }

    /// creates a new note given the difference in cents from a4
    pub const fn from_cent_difference(difference: i32) -> Self {
        Self(difference)
    }

    /// gets the difference in cents from a4
    pub const fn cent_difference(&self) -> i32 {
        self.0
    }

    /// gets the frequency of the note in 12-tone equal temperment
    pub fn frequency(&self, a4_frequency: f64) -> f64 {
        a4_frequency * 2f64.powf(self.0 as f64 / 1200f64)
    }
}

#[derive(Debug, Clone)]
pub enum NoteParseError {
    Overflow,
	MissingOctave,
    InvalidOctave,
    MissingTone,
    InvalidTone,
}

impl FromStr for Note {
    type Err = NoteParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // mutable iterator over characters in the string
        let mut chars = s.chars().peekable();

        // the character representing the base tone
        let tone_base = chars.next();
        if tone_base.is_none() {
            return Err(NoteParseError::MissingTone);
        }
        let tone_base = tone_base.unwrap();

        // the difference in cents from c4
        let mut c_delta: i64 = match tone_base.to_ascii_lowercase() {
            'c' => 0,
            'd' => 200,
            'e' => 400,
            'f' => 500,
            'g' => 700,
            'a' => 900,
            'b' => 1100,
            _ => { return Err(NoteParseError::InvalidOctave); }
        };

        // the sign of the octave
        let mut signum: i64 = 1;

        // the magnitude of the octave
        let mut octave: i64 = 0;

        // handle parsing the accidental
        let potential_accidental = chars.peek();
        if potential_accidental.is_none() {
            return Err(NoteParseError::MissingOctave);
        }
        let potential_accidental = potential_accidental.unwrap();
        if *potential_accidental == 'b' {
            c_delta -= 100;
            chars.next();
        } else {
            c_delta += 100;
            chars.next();
        }

        // handle parsing a negative sign on the octave
        let potential_negative = chars.peek();
        if potential_negative.is_none() {
            return Err(NoteParseError::MissingOctave);
        }
        let potential_negative = potential_negative.unwrap();
        if *potential_negative == '-' {
            signum = -1;
            chars.next();
        }

        // check that there are characters used for the octave
        if chars.peek().is_none() {
            return Err(NoteParseError::MissingOctave);
        }

        // parse the octave
        while let Some(char) = chars.next() {
            if let Some(digit) = char.to_digit(10) {
                octave *= 10;
                octave += digit as i64;
                if i32::try_from(octave).is_err() {
                    return Err(NoteParseError::Overflow);
                }
            } else {
                return Err(NoteParseError::InvalidOctave);
            }
        }

        // calculate difference from a4
        let a4_delta_i64 = signum * octave * 1200 - 4800 + c_delta - 900;
        if let Ok(a4_delta) = i32::try_from(a4_delta_i64) {
            Ok(Note(a4_delta))
        } else {
            Err(NoteParseError::Overflow)
        }
    }
}

