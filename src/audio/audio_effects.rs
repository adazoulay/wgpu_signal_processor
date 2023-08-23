// // use dasp::Frame;
// use std::cell::Cell;

// trait AudioEffect<F: Frame> {
//     fn apply_effect(&self, frame: &mut F);
// }

// struct Gain {
//     factor: f32,
// }

// impl AudioEffect<dasp::frame::Mono<f32>> for Gain {
//     fn apply_effect(&self, frame: &mut dasp::frame::Mono<f32>) {
//         frame.mul_amp(self.factor);
//     }
// }

// impl AudioEffect<dasp::frame::Stereo<f32>> for Gain {
//     fn apply_effect(&self, frame: &mut dasp::frame::Stereo<f32>) {
//         frame.mul_amp(self.factor);
//     }
// }

// struct Invert;

// impl AudioEffect<dasp::frame::Mono<f32>> for Invert {
//     fn apply_effect(&self, frame: &mut dasp::frame::Mono<f32>) {
//         frame.scale_amp(-1.0);
//     }
// }

// impl AudioEffect<dasp::frame::Stereo<f32>> for Invert {
//     fn apply_effect(&self, frame: &mut dasp::frame::Stereo<f32>) {
//         frame.scale_amp(-1.0);
//     }
// }

// struct FadeIn {
//     duration: usize,
//     counter: Cell<usize>,
// }

// impl AudioEffect<dasp::frame::Mono<f32>> for FadeIn {
//     fn apply_effect(&self, frame: &mut dasp::frame::Mono<f32>) {
//         let fade_factor = (self.counter.get() as f32) / (self.duration as f32);
//         frame.scale_amp(fade_factor.min(1.0));
//         self.counter.set(self.counter.get() + 1);
//     }
// }

// impl AudioEffect<dasp::frame::Stereo<f32>> for FadeIn {
//     fn apply_effect(&self, frame: &mut dasp::frame::Stereo<f32>) {
//         let fade_factor = (self.counter.get() as f32) / (self.duration as f32);
//         frame.scale_amp(fade_factor.min(1.0));
//         self.counter.set(self.counter.get() + 1);
//     }
// }

// pub type EffectFn<F> = Box<dyn AudioEffect<F> + Send + Sync>;

// pub fn gain<F: Frame<Sample = f32>>(factor: f32) -> EffectFn<F>
// where
//     F: AudioEffect<dasp::frame::Mono<f32>> + AudioEffect<dasp::frame::Stereo<f32>>,
// {
//     Box::new(Gain { factor })
// }

// pub fn invert<F: Frame<Sample = f32>>() -> EffectFn<F>
// where
//     F: AudioEffect<dasp::frame::Mono<f32>> + AudioEffect<dasp::frame::Stereo<f32>>,
// {
//     Box::new(Invert)
// }

// pub fn fade_in<F: Frame<Sample = f32>>(duration: usize) -> EffectFn<F>
// where
//     F: AudioEffect<dasp::frame::Mono<f32>> + AudioEffect<dasp::frame::Stereo<f32>>,
// {
//     Box::new(FadeIn {
//         duration,
//         counter: Cell::new(0),
//     })
// }
