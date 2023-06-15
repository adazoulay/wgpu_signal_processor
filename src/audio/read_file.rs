use minimp3::{Decoder as Mp3Decoder, Frame};
use rodio::{source::Source, Decoder, OutputStream};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FftPlanner;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub struct FileInfo {
    pub samples: Vec<f32>,
    pub size: usize,
    pub sample_rate: u32,
    pub max_amplitude: f32,
    pub slice_size: usize,
}

impl FileInfo {
    pub fn new() -> Self {
        let (samples, sample_rate, max_amplitude) = get_file();
        let size = samples.len();

        let slice_size = get_sice_size(sample_rate as f32, 60.0);

        FileInfo {
            samples,
            size,
            sample_rate,
            max_amplitude,
            slice_size,
        }
    }

    pub fn get_table(&self) -> Vec<[f32; 2]> {
        let scale_factor = 1.0 / self.max_amplitude;
        let num_slices = self.samples.len() / self.slice_size;
        println!("NUMBER OF SLICES {}", num_slices);
        let fft = FftPlanner::new().plan_fft_forward(self.slice_size);

        let mut table = Vec::with_capacity(num_slices * self.slice_size);

        for slice_idx in 0..num_slices {
            let start = slice_idx * self.slice_size;
            let end = start + self.slice_size;
            let slice = &self.samples[start..end];

            // Apply FFT to the slice
            let mut fft_input: Vec<Complex<f32>> =
                slice.iter().map(|x| Complex::new(*x, 0.0)).collect();
            let fft_output = fft_input.clone();
            fft.process(&mut fft_input);

            // Normalize the FFT results and convert the data
            for (i, x) in fft_output.iter().enumerate() {
                let x_component = -1.0 + (2.0 * i as f32 / (self.slice_size - 1) as f32);
                let y_component = x.norm() * scale_factor;
                table.push([x_component, y_component]);
            }
        }

        table
    }
}

pub fn get_file() -> (Vec<f32>, u32, f32) {
    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src");
    path.push("audio");
    path.push("test.mp3");

    let mut mp3_data = Vec::new();
    File::open(&path)
        .unwrap()
        .read_to_end(&mut mp3_data)
        .unwrap();
    let mut mp3_decoder = Mp3Decoder::new(mp3_data.as_slice());

    let mut samples = Vec::new();
    let mut max_amplitude = f32::NEG_INFINITY;
    let mut sample_rate = 0;

    while let Ok(Frame {
        data,
        sample_rate: sr,
        ..
    }) = mp3_decoder.next_frame()
    {
        sample_rate = sr;
        for sample in data {
            let f = sample as f32;
            max_amplitude = max_amplitude.max(f.abs());
            samples.push(f);
        }
    }

    (samples, sample_rate as u32, max_amplitude)
}

pub fn get_sice_size(sample_rate: f32, frame_rate: f32) -> usize {
    return (sample_rate / frame_rate) as usize;
}

impl FileInfo {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}
