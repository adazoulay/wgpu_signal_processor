pub enum WaveType {
    Sine,
    Sawtooth,
    Triangle,
    Square,
    Pulse,
}

pub struct WaveTable {
    pub size: usize,
    pub table: Vec<f32>,
    wave_type: WaveType,
}

impl WaveTable {
    pub fn new(size: usize, wave_type: WaveType) -> Self {
        let table: Vec<f32> = (0..size)
            .map(|n| match wave_type {
                WaveType::Sine => (2.0 * std::f32::consts::PI * (n as f32) / (size as f32)).sin(),
                WaveType::Sawtooth => 2.0 * (n as f32 / size as f32) - 1.0,
                WaveType::Triangle => {
                    let x = n as f32 / size as f32;
                    if x < 0.5 {
                        4.0 * x - 1.0
                    } else {
                        -4.0 * x + 3.0
                    }
                }
                WaveType::Square => {
                    if n < size / 2 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                WaveType::Pulse => {
                    if n < size / 4 || n > 3 * size / 4 {
                        1.0
                    } else {
                        -1.0
                    }
                }
            })
            .collect();

        Self {
            size,
            table,
            wave_type,
        }
    }
}

pub struct WaveTableOscillator {
    sample_rate: u32,
    wave_table: WaveTable,
    index: f32,
    index_increment: f32,
}

impl WaveTableOscillator {
    pub fn new(sample_rate: u32, wave_table: WaveTable) -> Self {
        return WaveTableOscillator {
            sample_rate,
            wave_table,
            index: 0.0,
            index_increment: 0.0,
        };
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.index_increment =
            frequency * (self.wave_table.size as f32) / (self.sample_rate as f32);
    }

    fn get_sample(&mut self) -> f32 {
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.size as f32;
        return sample;
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.size;

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        return truncated_index_weight * self.wave_table.table[truncated_index]
            + next_index_weight * self.wave_table.table[next_index];
    }
}

// ! Trait Implemenations

impl Iterator for WaveTableOscillator {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_sample());
    }
}

use rodio::Source;

impl Source for WaveTableOscillator {
    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate;
    }

    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        return None;
    }
}

impl WaveTableOscillator {
    pub fn get_raw_normalized_table(&self) -> Vec<[f32; 2]> {
        let mut table = Vec::with_capacity(self.wave_table.size);
        let mut max_amplitude = f32::NEG_INFINITY;

        for (i, x) in self.wave_table.table.iter().enumerate() {
            let x_component = -1.0 + (2.0 * i as f32 / (self.wave_table.size - 1) as f32);
            let y_component = *x;
            let amplitude = y_component.abs();
            if amplitude > max_amplitude {
                max_amplitude = amplitude;
            }
            table.push([x_component, y_component]);
        }

        let scale_factor = 0.8 / max_amplitude;

        for point in table.iter_mut() {
            point[1] *= scale_factor;
        }

        table
    }

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
