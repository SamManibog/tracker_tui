use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Note(i32);

impl Note {
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
