mod audio_glue;
use rifgen::rifgen_attr::*;

pub use crate::audio_glue::*;

use atomic_float::AtomicF32;
use std::{
    f32::consts::PI,
    marker::PhantomData,
    sync::{atomic::Ordering, Arc},
};

use oboe::{
    AudioOutputCallback, AudioOutputStream, AudioOutputStreamSafe, AudioStream, AudioStreamAsync,
    AudioStreamBase, AudioStreamBuilder, DataCallbackResult, Mono, Output, PerformanceMode,
    SharingMode, Stereo,
};

/// Sine-wave generator stream
#[derive(Default)]
#[generate_interface_doc]
pub struct AudioSineWaveGen {
    stream: Option<AudioStreamAsync<Output, SineWave<f32, Mono>>>,
}

impl AudioSineWaveGen {
    #[generate_interface(constructor)]
    pub fn new() -> AudioSineWaveGen {
        AudioSineWaveGen { stream: None }
    }

    /// Create and start audio stream
    #[generate_interface]
    pub fn try_start(&mut self) {
        if self.stream.is_none() {
            let param = Arc::new(SineParam::default());

            let mut stream = AudioStreamBuilder::default()
                .set_performance_mode(PerformanceMode::LowLatency)
                .set_sharing_mode(SharingMode::Shared)
                .set_format::<f32>()
                .set_channel_count::<Mono>()
                .set_callback(SineWave::<f32, Mono>::new(&param))
                .open_stream()
                .unwrap();

            log::debug!("start stream: {:?}", stream);

            param.set_sample_rate(stream.get_sample_rate() as _);

            stream.start().unwrap();

            self.stream = Some(stream);
        }
    }

    /// Pause audio stream
    #[generate_interface]
    pub fn try_pause(&mut self) {
        if let Some(stream) = &mut self.stream {
            log::debug!("pause stream: {:?}", stream);
            stream.pause().unwrap();
        }
    }

    /// Stop and remove audio stream
    #[generate_interface]
    pub fn try_stop(&mut self) {
        if let Some(stream) = &mut self.stream {
            log::debug!("stop stream: {:?}", stream);
            stream.stop().unwrap();
            self.stream = None;
        }
    }
    /// Print device's audio info
    #[generate_interface]
    pub fn audio_probe() {
        // if let Err(error) = DefaultStreamValues::init() {
        //     log::error!("Unable to init default stream values due to: {}", error);
        // }

        // log::debug!("Default stream values:");
        // log::debug!("  Sample rate: {}", DefaultStreamValues::get_sample_rate());
        // log::debug!(
        //     "  Frames per burst: {}",
        //     DefaultStreamValues::get_frames_per_burst()
        // );
        // log::debug!(
        //     "  Channel count: {}",
        //     DefaultStreamValues::get_channel_count()
        // );

        // log::debug!("Audio features:");
        // log::debug!("  Low latency: {}", AudioFeature::LowLatency.has().unwrap());
        // log::debug!("  Output: {}", AudioFeature::Output.has().unwrap());
        // log::debug!("  Pro: {}", AudioFeature::Pro.has().unwrap());
        // log::debug!("  Microphone: {}", AudioFeature::Microphone.has().unwrap());
        // log::debug!("  Midi: {}", AudioFeature::Midi.has().unwrap());

        // let devices = AudioDeviceInfo::request(AudioDeviceDirection::InputOutput).unwrap();

        // log::debug!("Audio Devices:");

        // for device in devices {
        //     log::debug!("{{");
        //     log::debug!("  Id: {}", device.id);
        //     log::debug!("  Type: {:?}", device.device_type);
        //     log::debug!("  Direction: {:?}", device.direction);
        //     log::debug!("  Address: {}", device.address);
        //     log::debug!("  Product name: {}", device.product_name);
        //     log::debug!("  Channel counts: {:?}", device.channel_counts);
        //     log::debug!("  Sample rates: {:?}", device.sample_rates);
        //     log::debug!("  Formats: {:?}", device.formats);
        //     log::debug!("}}");
        // }
    }
}

pub struct SineParam {
    frequency: AtomicF32,
    gain: AtomicF32,
    sample_rate: AtomicF32,
    delta: AtomicF32,
}

impl Default for SineParam {
    fn default() -> Self {
        Self {
            frequency: AtomicF32::new(440.0),
            gain: AtomicF32::new(0.5),
            sample_rate: AtomicF32::new(0.0),
            delta: AtomicF32::new(0.0),
        }
    }
}

impl SineParam {
    fn set_sample_rate(&self, sample_rate: f32) {
        let frequency = self.frequency.load(Ordering::Acquire);
        let delta = frequency * 2.0 * PI / sample_rate;

        self.delta.store(delta, Ordering::Release);
        self.sample_rate.store(sample_rate, Ordering::Relaxed);

        log::debug!(
            "Prepare sine wave generator: samplerate={}, time delta={}",
            sample_rate,
            delta
        );
    }

    fn set_frequency(&self, frequency: f32) {
        let sample_rate = self.sample_rate.load(Ordering::Relaxed);
        let delta = frequency * 2.0 * PI / sample_rate;

        self.delta.store(delta, Ordering::Relaxed);
        self.frequency.store(frequency, Ordering::Relaxed);
    }

    fn set_gain(&self, gain: f32) {
        self.gain.store(gain, Ordering::Relaxed);
    }
}

pub struct SineWave<F, C> {
    param: Arc<SineParam>,
    phase: f32,
    marker: PhantomData<(F, C)>,
}

impl<F, C> Drop for SineWave<F, C> {
    fn drop(&mut self) {
        log::debug!("drop SineWave generator");
    }
}

impl<F, C> SineWave<F, C> {
    pub fn new(param: &Arc<SineParam>) -> Self {
        log::debug!("init SineWave generator");
        Self {
            param: param.clone(),
            phase: 0.0,
            marker: PhantomData,
        }
    }
}

impl<F, C> Iterator for SineWave<F, C> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let delta = self.param.delta.load(Ordering::Relaxed);
        let gain = self.param.gain.load(Ordering::Relaxed);

        let frame = gain * self.phase.sin();

        self.phase += delta;
        while self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        Some(frame)
    }
}

impl AudioOutputCallback for SineWave<f32, Mono> {
    type FrameType = (f32, Mono);

    fn on_audio_ready(
        &mut self,
        stream: &mut dyn AudioOutputStreamSafe,
        frames: &mut [f32],
    ) -> DataCallbackResult {
        for frame in frames {
            *frame = self.next().unwrap();
        }
        DataCallbackResult::Continue
    }
}

impl AudioOutputCallback for SineWave<f32, Stereo> {
    type FrameType = (f32, Stereo);

    fn on_audio_ready(
        &mut self,
        stream: &mut dyn AudioOutputStreamSafe,
        frames: &mut [(f32, f32)],
    ) -> DataCallbackResult {
        for frame in frames {
            frame.0 = self.next().unwrap();
            frame.1 = frame.0;
        }
        DataCallbackResult::Continue
    }
}

// use jni::{JNIEnv, JavaVM};
// use std::ffi::c_void;

// #[no_mangle]
// pub extern "C" fn JNI_OnLoad(vm: jni::JavaVM, res: *mut std::os::raw::c_void) -> jni::sys::jint {
//     let env = vm.get_env().unwrap();
//     let vm = vm.get_java_vm_pointer() as *mut c_void;
//     unsafe {
//         ndk_context::initialize_android_context(vm, res);
//     }
//     jni::JNIVersion::V6.into()
// }
