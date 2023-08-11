use cpal::traits::{DeviceTrait, HostTrait};

use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};


pub struct AudioIO {
    _host: cpal::Host,
    // Output
    pub output_device: cpal::Device,
    pub supported_output_config: cpal::SupportedStreamConfig,
    // Input
    pub input_device: cpal::Device,
    pub supported_input_config: cpal::SupportedStreamConfig,
}

impl AudioIO {
    pub fn new() -> Self {
        let _host = cpal::default_host();

        // Output
        let output_device = _host.default_output_device().unwrap();
        let mut supported_configs_range = output_device
            .supported_output_configs()
            .expect("error while querying configs");

        let supported_output_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();

        // Input
        let input_device = _host.default_input_device().unwrap();

        let mut supported_configs_range = input_device
            .supported_input_configs()
            .expect("error while querying configs");

        let supported_input_config = supported_configs_range
            .next()
            .expect("no supported config")
            .with_max_sample_rate();

        Self {
            _host,
            output_device,
            supported_output_config,
            input_device,
            supported_input_config,
        }
    }

    pub fn record(&self) -> Option<(Vec<f32>, u32, u32)> {
    
        let clip = Vec::new();
        let clip = Arc::new(Mutex::new(Some(clip)));
        let clip_2 = Arc::clone(&clip);

        println!("Begin recording...");
        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let channels = self.supported_input_config.channels();
        let sample_rate = self.supported_input_config.sample_rate().0;

        type ClipHandle = Arc<Mutex<Option<Vec<f32>>>>;

        fn write_input_data<T>(input: &[T], channels: u16, writer: &ClipHandle)
        where
            T: cpal::Sample + Debug,
            f32: cpal::FromSample<T>,
        {
            if let Ok(mut guard) = writer.try_lock() {
                if let Some(clip) = guard.as_mut() {
                    for frame in input.chunks(channels.into()) {
                        clip.push(frame[0].to_sample::<f32>());
                    }
                }
            }
        }

        let stream = match self.supported_input_config.sample_format() {
            cpal::SampleFormat::F32 => self.input_device.build_input_stream(
                &self.supported_input_config.clone().into(),
                move |data, _: &_| write_input_data::<f32>(data, channels, &clip_2),
                err_fn,
                None,
            ),
            _ => unimplemented!(),
        };

        std::thread::sleep(std::time::Duration::from_secs(5));
        drop(stream);
        let clip = clip.lock().unwrap().take().unwrap();
        eprintln!("recorded clip: {:?}", clip.len());
        Some((clip, sample_rate, channels as u32))
    }
}


