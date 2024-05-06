use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig, BuildStreamError, Sample as CpalSample};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use anyhow::{anyhow, Error};
use serde_json::json;
use std::time::Duration;

#[derive(Serialize)]
struct EmptyResponse {}

#[derive(Serialize, Deserialize)]
struct Device {
    name: String,
}

impl Device {
    fn new(name: String) -> Self {
        Device { name }
    }
}

#[tauri::command]
async fn get_audio_devices() -> Result<String, anyhow::Error> {
    let host = cpal::default_host();
    let input_devices_future = async {
        host.input_devices()
            .map_err(|e| anyhow!(e))
            .and_then(|devices| {
                Ok(devices.map(|d| Device::new(d.name().unwrap())).collect::<Vec<_>>())
            })
    };
    let output_devices_future = async {
        host.output_devices()
            .map_err(|e| anyhow!(e))
            .and_then(|devices| {
                Ok(devices.map(|d| Device::new(d.name().unwrap())).collect::<Vec<_>>())
            })
    };

    let input_devices = input_devices_future.await?;
    let output_devices = output_devices_future.await?;

    Ok(serde_json::to_string(&json!({
        "input_devices": input_devices,
        "output_devices": output_devices
    }))?)
}


#[tauri::command]
fn start_audio_capture(device_name: String) -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let device = host.input_devices()?
        .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
        .ok_or_else(|| anyhow!("Device not found"))?;

    let config = device.default_input_config()?;

    let samples = Arc::new(Mutex::new(Vec::new()));

    let samples_clone = samples.clone();
    let err_fn = move |err| eprintln!("Stream error: {:?}", err);
    let stream = match config.sample_format() {
        SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), samples_clone, err_fn, None)?,
        SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), samples_clone, err_fn, None)?,
        SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), samples_clone, err_fn, None)?,
        _ => return Err(anyhow!("Unsupported sample format")),
    };

    stream.play()?;
    Ok(())
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + Sync + 'static,
    timeout: Option<Duration>,
) -> Result<cpal::Stream, BuildStreamError>
    where
        T: CpalSample + Send + 'static + cpal::SizedSample,
{
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut samples = samples.lock().unwrap();
            for &sample in data.iter() {
                samples.push(sample.to_f32()); // Correct usage
            }
        },
        err_fn,
        timeout,
    )
}


#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_audio_devices, start_audio_capture])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
