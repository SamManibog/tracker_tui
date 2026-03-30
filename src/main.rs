use std::{collections::{HashMap, HashSet}, str::FromStr, sync::{Arc, OnceLock, mpsc::{self, Receiver, Sender}}};

use cpal::{Stream, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use egui::Event;
use rsynth::{osc_synths::PolyphonicOscSynth, *};

fn main() {
    play_pattern();
}

// plays a fun little pattern lol
fn play_pattern() {
    static PATTERN: OnceLock<&'static Pattern> = OnceLock::new();

    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");

    let sample_rate = 48000;
    let supported_config = supported_configs_range
        .find(|config| config.try_with_sample_rate(sample_rate).is_some())
        .expect("no supported config?!")
        .with_sample_rate(sample_rate);
    let config: StreamConfig = supported_config.into();

    let synth = (PolyphonicOscSynth::sine_specification().generate_synth)();
    let &pattern_ref = PATTERN.get_or_init(|| {
        let mut pattern = Box::new(Pattern::new());
        let composition: &[(&[&str], PatternTime, PatternDuration)] = &[
            (
                &["d4", "f4", "g4", "c5"],
                PatternTime::from_subdivision_count(4, 0).unwrap(),
                PatternDuration::from_subdivision_count(4, 1).unwrap()
            ),
            (
                &["d4", "f4", "g4", "b4"],
                PatternTime::from_subdivision_count(4, 1).unwrap(),
                PatternDuration::from_subdivision_count(4, 1).unwrap()
            ),
            (
                &["c4", "e4", "a4", "b4", "d5"],
                PatternTime::from_subdivision_count(4, 2).unwrap(),
                PatternDuration::from_subdivision_count(4, 2).unwrap()
            ),
        ];
        for (notes, start, duration) in composition {
            let end = *start + *duration;
            for note in *notes {
                pattern.add_note(
                    Note::from_str(note).unwrap(),
                    *start,
                    end
                );
            }
        }
        Box::leak(pattern)
    });

    let mut pattern_player = PatternPlayer::new(
        (&pattern_ref).command_iter(),
        synth,
        0.2,
        sample_rate
    );

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            pattern_player.generate_samples(data);
        },
        move |_err| {
            println!("ERROR BRUH")
        },
        None // None=blocking, Some(Duration)=timeout
    ).expect("Could not build stream.");
    let _ = stream.play().expect("stream could not play");

    loop {}
}

/*
fn run_synth_test {
    let specification_map = Arc::new(HashMap::from([
        (0, Box::new(PolyphonicOscSynth::sine_specification())),
        (1, Box::new(PolyphonicOscSynth::square_specification())),
        (2, Box::new(PolyphonicOscSynth::saw_specification())),
    ]));

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "RSynth",
        native_options,
        Box::new(|cc| Ok(Box::new(RsynthApp::new(
            cc,
            specification_map
        ))))
    );
}

struct RsynthApp {
    pressed_notes: HashSet<Note>,
    sender: Sender<SynthManagerCommand>,
    reciever: Receiver<SynthManagerMessage>,

    #[allow(dead_code)]
    stream: Stream,
}

impl RsynthApp {
    pub fn new(
        _cc: &eframe::CreationContext,
        synth_specs: Arc<HashMap<u32, Box<SynthesizerSpecification>>>
    ) -> Self {
        let (command_sender, command_reciever) = mpsc::channel();
        let (mut message_sender, message_reciever) = mpsc::channel();
        let _ = command_sender.send(SynthManagerCommand::AddSynth{synth_type_id: 2});

        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        let mut supported_configs_range = device.supported_output_configs()
            .expect("error while querying configs");

        let sample_rate = 48000;
        let supported_config = supported_configs_range
            .find(|config| config.try_with_sample_rate(sample_rate).is_some())
            .expect("no supported config?!")
            .with_sample_rate(sample_rate);
        let config: StreamConfig = supported_config.into();

        let mut manager = SynthManager::new(synth_specs.clone());

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                while let Ok(command) = command_reciever.try_recv() {
                    manager.handle_command(command, &mut message_sender);
                }
                manager.generate_samples(data, sample_rate);
            },
            move |_err| {
                println!("ERROR BRUH")
            },
            None // None=blocking, Some(Duration)=timeout
        ).expect("Could not build stream.");
        let _ = stream.play().expect("stream could not play");

        Self {
            pressed_notes: HashSet::new(),
            sender: command_sender,
            reciever: message_reciever,
            stream
        }
    }

    pub fn key_to_note(key: &str) -> Option<Note> {
        let multiplier = if key == "Comma" {
            12
        } else {
            if key.len() != 1 {
                return None;
            }
            match key.chars().next().expect("key.len == 1") {
                'Z' => 0,
                'S' => 1,
                'X' => 2,
                'D' => 3,
                'C' => 4,
                'V' => 5,
                'G' => 6,
                'B' => 7,
                'H' => 8,
                'N' => 9,
                'J' => 10,
                'M' => 11,
                ',' => 12,
                'Q' => 12,
                '2' => 13,
                'W' => 14,
                '3' => 15,
                'E' => 16,
                'R' => 17,
                '5' => 18,
                'T' => 19,
                '6' => 20,
                'Y' => 21,
                '7' => 22,
                'U' => 23,
                'I' => 24,
                _ => return None,
            }
        };
        Some(Note::from_semitone_delta_a4(multiplier - 9))
    }
}

impl eframe::App for RsynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.label("hi")
            });
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        for event in _raw_input.events.iter() {
            if let Event::Key {
                key, 
                physical_key: _,
                pressed,
                repeat: _,
                modifiers: _
            } = event {
                let name = key.name();
                if let Some(note) = Self::key_to_note(name) {
                    if *pressed {
                        if self.pressed_notes.insert(note) {
                            let _ = self.sender.send(SynthManagerCommand::PlayNote{synth_id: SynthId(0), note});
                        }
                    } else {
                        if self.pressed_notes.remove(&note) {
                            let _ = self.sender.send(SynthManagerCommand::StopNote{synth_id: SynthId(0), note});
                        }
                    }
                }
            }
        }
    }
}
*/
