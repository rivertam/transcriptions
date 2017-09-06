extern crate cpal;
extern crate futures;
extern crate hound;

use cpal::{Sample};

fn main() {
    run()
}

fn run() {
    use futures::stream::Stream;

    let mut reader = hound::WavReader::open("test_data/1/phone.wav").unwrap();
    let phone: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    let mut iter = phone.into_iter();
    let wav_sample_rate = reader.spec().sample_rate;

    let endpoint = cpal::get_default_endpoint().expect("Failed to get endpoint.");

    let format = endpoint
        .get_supported_formats_list()
        .unwrap()
        .find(|&cpal::Format { samples_rate: cpal::SamplesRate(rate), ref channels, .. }| (rate == wav_sample_rate) && (channels.len() > 1))
        .expect("No valid output format found.");

    for f in endpoint.get_supported_formats_list().unwrap().filter(|&cpal::Format { samples_rate: cpal::SamplesRate(rate), .. }| rate == wav_sample_rate) {
        println!("{:?}", f);
    }


    struct Executor;
    impl futures::task::Executor for Executor {
        fn execute(&self, r: futures::task::Run) {
            r.run();
        }
    }

    let executor = std::sync::Arc::new(Executor);

    let event_loop = cpal::EventLoop::new();

    let (mut voice, stream) = cpal::Voice::new(&endpoint, &format, &event_loop).expect("Failed to create a voice");

    fn write_to_buffer<S, I>(mut buffer: cpal::Buffer<S>, channels: usize, phone: &mut I)
        where S: Sample,
              I: Iterator<Item=S>
    {
        match channels {
            1 => for (frame, phone_frame) in buffer.chunks_mut(channels).zip(phone) {
                frame[0] = phone_frame;
            },

            2 => for (frame, phone_sample) in buffer.chunks_mut(channels).zip(phone) {
                for sample in frame {
                    *sample = phone_sample;
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
                &mut iter.clone().map(|s| s as u16)
            ),
            cpal::UnknownTypeBuffer::I16(buffer) => write_to_buffer(
                buffer,
                format.channels.len(),
                &mut iter
            ),
            cpal::UnknownTypeBuffer::F32(buffer) => write_to_buffer(
                buffer,
                format.channels.len(),
                &mut iter.clone().map(|s| s as f32)
            ),
        };
        Ok(())
    })).execute(executor);

    std::thread::spawn(move || {
        voice.play();
    });

    event_loop.run();
}
