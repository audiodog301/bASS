use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;
mod dsp;

pub struct Bass {
    params: Arc<BassParams>,
    clip_gate: dsp::ClipGate,
}

#[derive(Params)]
struct BassParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "output gain"]
    pub output_gain: FloatParam,

    #[id = "skrunkle"]
    pub skrunkle: FloatParam,

    #[id = "threshold"]
    pub threshold: FloatParam,
}

impl Default for Bass {
    fn default() -> Self {
        Self {
            params: Arc::new(BassParams::default()),
            clip_gate: dsp::ClipGate::default(),
        }
    }
}

impl Default for BassParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            output_gain: FloatParam::new(
                "output gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            skrunkle: FloatParam::new(
                "skrunkle",
                1.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 10.00,
                    factor: FloatRange::gain_skew_factor(0.0, 10.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            threshold: FloatParam::new(
                "threshold",
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1.00,
                    factor: FloatRange::gain_skew_factor(0.0, 1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
        }
    }
}

impl Plugin for Bass {
    const NAME: &'static str = "bASS!~";
    const VENDOR: &'static str = "jeany's plugins";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut all_samples = Vec::new();

        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                all_samples.push(*sample);
            }
        }

        let rms = dsp::rms(all_samples);

        for channel_samples in buffer.iter_samples() {
            let num_samples = channel_samples.len();

            let gain = self.params.output_gain.smoothed.next();
            let skrunkle = self.params.skrunkle.smoothed.next();
            let threshold = self.params.threshold.smoothed.next(); // THIS RIGHT HERE IS THE PART THAT IS FUCKED

            self.clip_gate.set_mult(skrunkle);
            self.clip_gate.set_gate(threshold);

            for sample in channel_samples {
                *sample = self.clip_gate.process_sample(*sample, rms);
                *sample *= gain;
            }

            // To save resources, a plugin can (and probably should!) only perform expensive
            // calculations that are only displayed on the GUI while the GUI is open
            if self.params.editor_state.is_open() {
                //calculations
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Bass {
    const CLAP_ID: &'static str = "com.jeany.bASS!";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("makes the sound that much thiccer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Bass {
    const VST3_CLASS_ID: [u8; 16] = *b"RainGuiVIIIZIAAA";
    const VST3_CATEGORIES: &'static str = "Fx|Dynamics";
}

nih_export_clap!(Bass);
nih_export_vst3!(Bass);
