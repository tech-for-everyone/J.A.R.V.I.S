mod pvrecorder;

// mod cpal;
// mod portaudio;

use once_cell::sync::OnceCell;

use crate::{config, config::structs::RecorderType, DB};

static RECORDER_TYPE: OnceCell<RecorderType> = OnceCell::new();
static FRAME_LENGTH: OnceCell<u32> = OnceCell::new();

#[allow(clippy::result_unit_err)]
pub fn init() -> Result<(), ()> {
    // initialization is idempotent
    if RECORDER_TYPE.get().is_some() {
        return Ok(());
    }

    // set default recorder type
    // @TODO. Make it configurable?
    let recorder_type = config::DEFAULT_RECORDER_TYPE;
    RECORDER_TYPE
        .set(recorder_type)
        .map_err(|_| ())?;

    // some info
    info!("Loading recorder ...");
    info!("Available audio_devices are:\n{:?}", get_audio_devices());

    // load given recorder
    match recorder_type {
        RecorderType::PvRecorder => {
            // Init Pv Recorder
            info!("Initializing PvRecorder recording backend.");

            // pvrecorder requires frame buffer of 512
            FRAME_LENGTH
                .set(512u32)
                .map_err(|_| ())?;

            let selected_microphone = get_selected_microphone_index();
            let frame_length = FRAME_LENGTH.get().copied().unwrap_or(512);

            if !pvrecorder::init_microphone(selected_microphone, frame_length) {
                error!("Recorder initialization failed.");
                return Err(());
            }

            info!(
                "Recorder initialization success. Listening to microphone ({}): {}",
                selected_microphone,
                get_audio_device_name(selected_microphone)
            );
        }
        RecorderType::PortAudio => {
            // Init PortAudio
            error!("PortAudio recording backend is not implemented yet.");
            return Err(());
        }
        RecorderType::Cpal => {
            // Init CPAL
            error!("CPAL recording backend is not implemented yet.");
            return Err(());
        }
    }

    Ok(())
}

pub fn read_microphone(frame_buffer: &mut [i16]) {
    match RECORDER_TYPE.get() {
        Some(RecorderType::PvRecorder) => {
            pvrecorder::read_microphone(frame_buffer);
        }
        Some(RecorderType::PortAudio) | Some(RecorderType::Cpal) => {
            warn!("read_microphone called with an unsupported recorder backend");
        }
        None => {
            warn!("read_microphone called before the recorder was initialized");
        }
    }
}

#[allow(clippy::result_unit_err)]
pub fn start_recording() -> Result<(), ()> {
    match RECORDER_TYPE.get() {
        Some(RecorderType::PvRecorder) => {
            let frame_length = FRAME_LENGTH.get().copied().unwrap_or(512);
            pvrecorder::start_recording(get_selected_microphone_index(), frame_length)
        }
        Some(RecorderType::PortAudio) | Some(RecorderType::Cpal) => {
            error!("Recording is not supported with the selected backend");
            Err(())
        }
        None => {
            error!("start_recording called before the recorder was initialized");
            Err(())
        }
    }
}

#[allow(clippy::result_unit_err)]
pub fn stop_recording() -> Result<(), ()> {
    match RECORDER_TYPE.get() {
        Some(RecorderType::PvRecorder) => pvrecorder::stop_recording(),
        Some(RecorderType::PortAudio) | Some(RecorderType::Cpal) => {
            // nothing to stop
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn get_selected_microphone_index() -> i32 {
    let Some(settings) = DB.get() else {
        return -1;
    };
    let idx = settings.read().microphone;

    if idx > 0 {
        // validate that this microphone is actually in the list
        let devices = get_audio_devices();
        if (idx as usize) >= devices.len() {
            warn!(
                "Microphone index {} not found ({} available), falling back to default",
                idx,
                devices.len()
            );
            return -1;
        }
    }

    idx
}

pub fn get_audio_devices() -> Vec<String> {
    match RECORDER_TYPE.get() {
        Some(RecorderType::PvRecorder) => pvrecorder::list_audio_devices(),
        Some(RecorderType::PortAudio) | Some(RecorderType::Cpal) => {
            warn!("Audio device enumeration is not supported with the selected backend");
            Vec::new()
        }
        None => {
            // not initialized yet, default to pvrecorder
            pvrecorder::list_audio_devices()
        }
    }
}

pub fn get_audio_device_name(idx: i32) -> String {
    match RECORDER_TYPE.get() {
        Some(RecorderType::PvRecorder) => pvrecorder::get_audio_device_name(idx),
        Some(RecorderType::PortAudio) | Some(RecorderType::Cpal) => {
            warn!("Audio device names are not supported with the selected backend");
            String::from("Unknown Device")
        }
        None => {
            // not initialized yet, default to pvrecorder
            pvrecorder::get_audio_device_name(idx)
        }
    }
}
