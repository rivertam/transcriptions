extern crate cpal;
extern crate futures;
extern crate hound;

use cpal::{Sample};

fn main() {
    run()
}

fn run() {
    use futures::stream::Stream;

    let mut readers = vec![
        hound::WavReader::open("test_data/1/phone.wav").unwrap(),
        hound::WavReader::open("test_data/1/computer.wav").unwrap(),
    ];

    let wav_sample_rate = readers[0].spec().sample_rate;

    let endpoint = cpal::get_default_endpoint().expect("Failed to get endpoint.");
    let format = endpoint
        .get_supported_formats_list()
        .unwrap()
        .find(|&cpal::Format { samples_rate: cpal::SamplesRate(rate), ref channels, .. }| (rate == wav_sample_rate) && (channels.len() > 1))
        .expect("No valid output format found.");

    let samples: Vec<Vec<i16>> = readers
        .iter_mut()
        .map(|reader| -> Vec<i16> {
            reader.samples::<i16>().map(|s| s.unwrap()).collect()
        })
        .collect();

    let mut collated = samples
        .iter()
        .fold(vec![], |mut output: Vec<i16>, sample: &Vec<i16>| -> Vec<i16> {
            if sample.len() > output.len() {
                output.resize(sample.len(), 0);
            }

            let mut max: i16 = 0;
            for i in 0..sample.len() {
                let value = sample[i];

                let adjusted_value = value / (readers.len() as i16);
                if adjusted_value.abs() > max {
                    max = adjusted_value.abs();
                }

                output[i] += adjusted_value;
            }

            output
        })
        .into_iter();

    struct Executor;
    impl futures::task::Executor for Executor {
        fn execute(&self, r: futures::task::Run) {
            r.run();
        }
    }

    let executor = std::sync::Arc::new(Executor);

    let event_loop = cpal::EventLoop::new();

    let (mut voice, stream) = cpal::Voice::new(&endpoint, &format, &event_loop).expect("Failed to create a voice");

    fn write_to_buffer<S, I>(mut buffer: cpal::Buffer<S>, channels: usize, samples: &mut I)
        where S: Sample,
              I: Iterator<Item=S>
    {
        match channels {
            1 => for (frame, source_frame) in buffer.chunks_mut(channels).zip(samples) {
                frame[0] = source_frame;
            },

            2 => for (frame, source_frame) in buffer.chunks_mut(channels).zip(samples) {
                for sample in frame {
                    *sample = source_frame;
                }
            },

            _ => unimplemented!(),
        }
    }

    futures::task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
        match buffer {
            cpal::UnknownTypeBuffer::U16(buffer) => write_to_buffer(
                buffer,
                format.channels.len(),
                &mut collated.clone().map(|s| s as u16)
            ),
            cpal::UnknownTypeBuffer::I16(buffer) => write_to_buffer(
                buffer,
                format.channels.len(),
                &mut collated
            ),
            cpal::UnknownTypeBuffer::F32(buffer) => write_to_buffer(
                buffer,
                format.channels.len(),
                &mut collated.clone().map(|s| f32::from(s))
            ),
        };

        Ok(())
    })).execute(executor);

    std::thread::spawn(move || {
        voice.play();
    });

    event_loop.run();
}
