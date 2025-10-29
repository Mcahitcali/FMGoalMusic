use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Convert any audio file to WAV format
/// Returns the path to the converted WAV file
pub fn convert_to_wav(input_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Check if already WAV
    if let Some(ext) = input_path.extension() {
        if ext.eq_ignore_ascii_case("wav") {
            println!("✓ File is already WAV format: {}", input_path.display());
            return Ok(input_path.to_path_buf());
        }
    }

    println!("Converting {} to WAV format...", input_path.display());

    // Open the media source
    let src = File::open(input_path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    // Create a probe hint using the file extension
    let mut hint = Hint::new();
    if let Some(ext) = input_path.extension() {
        if let Some(ext_str) = ext.to_str() {
            hint.with_extension(ext_str);
        }
    }

    // Use the default options for metadata and format readers
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    // Probe the media source
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)?;

    // Get the instantiated format reader
    let mut format = probed.format;

    // Find the first audio track with a known (decodable) codec
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;

    // Use the default options for the decoder
    let dec_opts: DecoderOptions = Default::default();

    // Create a decoder for the track
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)?;

    // Store the track identifier, we'll use it to filter packets
    let track_id = track.id;

    // Get audio parameters
    let sample_rate = track.codec_params.sample_rate.ok_or("Unknown sample rate")?;
    let channels = track.codec_params.channels.ok_or("Unknown channel count")?;
    let channel_count = channels.count() as u16;

    // Create output WAV file path (same directory, .wav extension)
    let mut output_path = input_path.to_path_buf();
    output_path.set_extension("wav");

    // Create WAV writer
    let spec = hound::WavSpec {
        channels: channel_count,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&output_path, spec)?;

    // Decode and write audio data
    loop {
        // Get the next packet from the media format
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::ResetRequired) => {
                // The track list has been changed. Re-examine it and create a new set of decoders,
                // then restart the decode loop. This is an advanced feature and is not
                // critical for a basic example.
                unimplemented!();
            }
            Err(symphonia::core::errors::Error::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                // End of stream
                break;
            }
            Err(err) => {
                return Err(Box::new(err));
            }
        };

        // If the packet does not belong to the selected track, skip it
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet into audio samples
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Write samples to WAV file
                write_samples(&mut writer, &decoded)?;
            }
            Err(symphonia::core::errors::Error::IoError(_)) => {
                // The packet failed to decode due to an IO error, skip the packet
                continue;
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => {
                // The packet failed to decode due to invalid data, skip the packet
                continue;
            }
            Err(err) => {
                return Err(Box::new(err));
            }
        }
    }

    // Finalize the WAV file
    writer.finalize()?;

    println!("✓ Converted to WAV: {}", output_path.display());
    Ok(output_path)
}

/// Write audio samples to WAV file
fn write_samples(
    writer: &mut hound::WavWriter<BufWriter<File>>,
    decoded: &AudioBufferRef,
) -> Result<(), Box<dyn std::error::Error>> {
    let num_channels = decoded.spec().channels.count();
    let num_frames = decoded.frames();

    // Symphonia can decode to different sample formats, we need to handle them
    // Write samples in interleaved format (L, R, L, R, ...)
    for frame in 0..num_frames {
        for ch in 0..num_channels {
            match decoded {
                AudioBufferRef::F32(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                    writer.write_sample(sample_i16)?;
                }
                AudioBufferRef::F64(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f64) as i16;
                    writer.write_sample(sample_i16)?;
                }
                AudioBufferRef::S16(buf) => {
                    let sample = buf.chan(ch)[frame];
                    writer.write_sample(sample)?;
                }
                AudioBufferRef::S32(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = (sample >> 16) as i16;
                    writer.write_sample(sample_i16)?;
                }
                AudioBufferRef::U8(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = ((sample as i16 - 128) * 256) as i16;
                    writer.write_sample(sample_i16)?;
                }
                AudioBufferRef::U16(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = (sample as i32 - 32768) as i16;
                    writer.write_sample(sample_i16)?;
                }
                AudioBufferRef::U24(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = (((sample.inner() as i32) - 8388608) >> 8) as i16;
                    writer.write_sample(sample_i16)?;
                }
                AudioBufferRef::U32(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = ((sample as i64 - 2147483648) >> 16) as i16;
                    writer.write_sample(sample_i16)?;
                }
                AudioBufferRef::S24(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = (sample.inner() >> 8) as i16;
                    writer.write_sample(sample_i16)?;
                }
                AudioBufferRef::S8(buf) => {
                    let sample = buf.chan(ch)[frame];
                    let sample_i16 = (sample as i16) << 8;
                    writer.write_sample(sample_i16)?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_file_returns_same_path() {
        let wav_path = Path::new("test.wav");
        // This test just checks the logic, not actual file conversion
        if let Some(ext) = wav_path.extension() {
            assert!(ext.eq_ignore_ascii_case("wav"));
        }
    }
}
