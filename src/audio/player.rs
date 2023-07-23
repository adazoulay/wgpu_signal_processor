// use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// use std::collections::VecDeque;
// use std::sync::{Arc, Mutex};

// fn setup() {
//     let mut ring_buffer = Arc::new(Mutex::new(VecDeque::new()));
//     let host = cpal::default_host();
//     let device = host.default_output_device().unwrap();
//     let config = device.default_output_config().unwrap();

//     match config.sample_format() {
//         cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), Arc::clone(&ring_buffer)),
//         _ => unimplemented!(),
//     }
// }

// fn run<T: cpal::Sample>(
//     device: &cpal::Device,
//     config: &cpal::StreamConfig,
//     ring_buffer: Arc<Mutex<VecDeque<f32>>>,
// ) {
//     let stream = device
//         .build_output_stream(
//             config,
//             move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
//                 // Fill 'data' with audio samples and also write them to the ring buffer.
//                 for sample in data.iter_mut() {
//                     let value = next_sample(); // Generate the next audio sample.
//                     *sample = value;
//                     ring_buffer.lock().unwrap().push_back(value);
//                 }
//             },
//             |err| eprintln!("an error occurred on stream: {}", err),
//             None,
//         )
//         .unwrap();

//     stream.play().unwrap();

//     // Start the visualization thread.
//     std::thread::spawn(move || {
//         loop {
//             let samples = ring_buffer.lock().unwrap().drain(..).collect::<Vec<_>>();
//             // Update the visualization with 'samples'.
//         }
//     });

//     // Keep the main thread alive until you want to stop.
//     loop {
//         std::thread::sleep(std::time::Duration::from_secs(1));
//     }
// }

// // struct Player {
// //     host: cpal::Host,
// //     device: cpal::Device,
// //     config: SupportedStreamConfig,
// // }

// // impl Player {
// //     pub fn new() -> Self {
// //         let host = cpal::default_host();
// //         let device = host.default_output_device().unwrap();
// //         let config = device.default_output_config().unwrap();

// //         Self {
// //             host,
// //             device,
// //             config,
// //         }
// //     }

// //     pub fn play(self, ring_buffer: Arc<Mutex<VecDeque<f32>>>) {
// //         match self.config.sample_format() {
// //             cpal::SampleFormat::F32 => {
// //                 run::<f32>(&self.device, &self.config.into(), Arc::clone(&ring_buffer))
// //             }
// //             _ => unimplemented!(),
// //         }
// //     }
// // }

// // fn run<T: cpal::Sample>(
// //     device: &cpal::Device,
// //     config: &cpal::StreamConfig,
// //     ring_buffer: Arc<Mutex<VecDeque<T>>>,
// // ) {
// //     let stream = device
// //         .build_output_stream(
// //             config,
// //             move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
// //                 // Fill 'data' with audio samples and also write them to the ring buffer.
// //                 for sample in data.iter_mut() {
// //                     let value = next_sample(); // Generate the next audio sample.
// //                     *sample = value;
// //                     ring_buffer.lock().unwrap().push_back(value);
// //                 }
// //             },
// //             |err| eprintln!("an error occurred on stream: {}", err),
// //         )
// //         .unwrap();

// //     stream.play().unwrap();

// //     // Start the visualization thread.
// //     std::thread::spawn(move || {
// //         loop {
// //             let samples = ring_buffer.lock().unwrap().drain(..).collect::<Vec<_>>();
// //             // Update the visualization with 'samples'.
// //         }
// //     });

// //     // Keep the main thread alive until you want to stop.
// //     loop {
// //         std::thread::sleep(std::time::Duration::from_secs(1));
// //     }
// // }
