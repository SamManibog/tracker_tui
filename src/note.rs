use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Note(i32);

impl Note {
    /// the highest note
    pub const MAX: Self = Note(i32::MAX);

    /// the lowest note
    pub const MIN: Self = Note(i32::MIN);

    /// a4 (tuning note)
    pub const A4: Self = Note(0);

    /// c4 (middle C)
    pub const C4: Self = Note(-900);

    /// the lowest possible single-digit octave note
    pub const C0: Self = Self::from_semitone_delta_a4(-9 - 4 * 12);

    /// the highest possible single-digit octave note
    pub const B9: Self = Self::from_semitone_delta_a4(2 + 5 * 12);

    /// creates a new note given the difference in semitones from a4
    pub const fn from_semitone_delta_a4(difference: i32) -> Self {
        Self(difference * 100)
    }

    /// creates a new note given the difference in cents from a4
    pub const fn from_cent_delta_a4(difference: i32) -> Self {
        Self(difference)
    }

    /// gets the difference in cents from a4
    pub const fn cent_delta_a4(&self) -> i32 {
        self.0
    }

    /// adds the given number of cents to the frequency
    pub fn add_cents(self, delta: i32) -> Self {
        Self(self.0 + delta)
    }

    /// adds the given number of semitones to the frequency
    pub fn add_semitones(self, delta: i32) -> Self {
        Self(self.0 + delta * 100)
    }

    /// adds the given number of cents to the frequency using overflow checking
    pub fn checked_add_cents(self, delta: i32) -> Option<Self> {
        Some(Self(self.0.checked_add(delta)?))
    }

    /// adds the given number of semitones to the frequency using overflow checking
    pub fn checked_add_semitones(self, delta: i32) -> Option<Self> {
        Some(Self(self.0.checked_add(delta.checked_mul(100)?)?))
    }

    /// adds the given number of cents to the frequency, saturating at numeric bounds
    pub fn saturating_add_cents(self, delta: i32) -> Self {
        Self(self.0.saturating_add(delta))
    }

    /// adds the given number of semitones to the frequency, saturating at numeric bounds
    pub fn saturating_add_semitones(self, delta: i32) -> Self {
        Self(self.0.saturating_add(delta.saturating_mul(100)))
    }

    /// gets the frequency of the note in 12-tone equal temperment
    pub fn frequency(&self, a4_frequency: f64) -> f64 {
        a4_frequency * 2f64.powf(self.0 as f64 / 1200f64)
    }

    /// gets the difference in cents from the nearest semitone (C, C#, D, ...)
    pub fn detune(&self) -> i32 {
        let raw_cents = self.0.rem_euclid(100);
        if raw_cents <= 50 {
            raw_cents
        } else {
            raw_cents - 100
        }
    }

    /// writes the note in a right-padded character format using sharps for accidentals
    pub fn to_padded_string_sharps(&self) -> String {
        self.to_string_base(|semitone_delta| {
            match semitone_delta.rem_euclid(12) {
                0 => "C-",
                1 => "C#",
                2 => "D-",
                3 => "D#",
                4 => "E-",
                5 => "F-",
                6 => "F#",
                7 => "G-",
                8 => "G#",
                9 => "A-",
                10 => "A#",
                11 => "B-",
                _ => unreachable!("numbers mod 12 are in [0, 12)")
            }.to_string()
        })
    }

    /// writes the note in a right-padded character format using flats for accidentals
    pub fn to_padded_string_flats(&self) -> String {
        self.to_string_base(|semitone_delta| {
            match semitone_delta.rem_euclid(12) {
                0 => "C-",
                1 => "Db",
                2 => "D-",
                3 => "Eb",
                4 => "E-",
                5 => "F-",
                6 => "Gb",
                7 => "G-",
                8 => "Ab",
                9 => "A-",
                10 => "Bb",
                11 => "B-",
                _ => unreachable!("numbers mod 12 are in [0, 12)")
            }.to_string()
        })
    }

    /// writes the note as a string, using sharps for accidentals
    pub fn to_string_sharps(&self) -> String {
        self.to_string_base(|semitone_delta| {
            match semitone_delta.rem_euclid(12) {
                0 => "C",
                1 => "C#",
                2 => "D",
                3 => "D#",
                4 => "E",
                5 => "F",
                6 => "F#",
                7 => "G",
                8 => "G#",
                9 => "A",
                10 => "A#",
                11 => "B",
                _ => unreachable!("numbers mod 12 are in [0, 12)")
            }.to_string()
        })
    }

    /// writes the note as a string, using flats for accidentals
    pub fn to_string_flats(&self) -> String {
        self.to_string_base(|semitone_delta| {
            match semitone_delta.rem_euclid(12) {
                0 => "C",
                1 => "Db",
                2 => "D",
                3 => "Eb",
                4 => "E",
                5 => "F",
                6 => "Gb",
                7 => "G",
                8 => "Ab",
                9 => "A",
                10 => "Bb",
                11 => "B",
                _ => unreachable!("numbers mod 12 are in [0, 12)")
            }.to_string()
        })
    }

    /// writes the note as a string using the given function to map
    /// the given semitones away from C4 to a string
    fn to_string_base(&self, map_semitone: impl Fn(i32) -> String) -> String {
        // the difference in cents from the base semitone
    	let detune = self.detune();

        // the difference in semitones from c4
        let semitone_delta = ((self.0 - detune) / 100) + 9;

        // the string representation of the tone
        let tone = (map_semitone)(semitone_delta);

        // the octave
        let octave = semitone_delta.div_euclid(12) + 4;

        // the string represention of the detune
        let string_detune = if detune == 0 {
            "".to_string()
        } else {
            if detune.is_negative() {
                format!(" -{}¢", detune.abs())
            } else {
                format!(" +{}¢", detune)
            }
        };

        format!("{}{}{}", tone, octave, string_detune)
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
            _ => {
                return Err(NoteParseError::InvalidTone);
            }
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
        } else if *potential_accidental == '#' {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_equality() {
        for i in 0..200 {
            assert_eq!(
            	Note::from_cent_delta_a4(i),
                Note::from_cent_delta_a4(i),
        	);
        }
    }

    #[test]
    fn cent_difference_a4_create_and_check() {
        for i in 0..200 {
            assert_eq!(
                Note::from_cent_delta_a4(i).cent_delta_a4(),
                i,
            );
        }
    }

    #[test]
    fn parse_sharp_one_digit_octaves() {
        let pairs = [
            ("a#4", 0 + 1),
            ("c#2", -9 + -2 * 12 + 1),
            ("f#8", -4 + 4 * 12 + 1),
            ("b#0", 2 + -4 * 12 + 1),
            ("d#9", -7 + 5 * 12 + 1),
            ("e#1", -5 + -3 * 12 + 1),
            ("g#7", -2 + 3 * 12 + 1),
         ];

        for (string, semitone_delta) in &pairs {
            let expect_err = format!("{} should be parsable.", string);
            assert_eq!(
            	Note::from_str(string).expect(&expect_err),
                Note::from_semitone_delta_a4(*semitone_delta),
                "failed case {string}"
        	);
        }
    }

    #[test]
    fn parse_flat_one_digit_octaves() {
        let pairs = [
            ("ab4", 0 - 1),
            ("cb2", -9 + -2 * 12 - 1),
            ("fb8", -4 + 4 * 12 - 1),
            ("bb0", 2 + -4 * 12 - 1),
            ("db9", -7 + 5 * 12 - 1),
            ("eb1", -5 + -3 * 12 - 1),
            ("gb7", -2 + 3 * 12 - 1),
         ];

        for (string, semitone_delta) in &pairs {
            let expect_err = format!("{} should be parsable.", string);
            assert_eq!(
            	Note::from_str(string).expect(&expect_err),
                Note::from_semitone_delta_a4(*semitone_delta),
                "failed case {string}"
        	);
        }
    }

    #[test]
    fn parse_natural_one_digit_octaves() {
        let pairs = [
            ("a4", 0),
            ("c2", -9 + -2 * 12),
            ("f8", -4 + 4 * 12),
            ("b0", 2 + -4 * 12),
            ("d9", -7 + 5 * 12),
            ("e1", -5 + -3 * 12),
            ("g7", -2 + 3 * 12),
        ];

        for (string, semitone_delta) in &pairs {
            let expect_err = format!("{} should be parsable.", string);
            assert_eq!(
            	Note::from_str(string).expect(&expect_err),
                Note::from_semitone_delta_a4(*semitone_delta),
                "failed case {string}"
        	);
        }
    }
}
