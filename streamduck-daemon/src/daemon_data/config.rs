//! Requests related to configs
use std::io::Read;
use std::ops::Deref;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use streamduck_core::config::{ConfigError, DeviceConfig};
use streamduck_core::core::methods::{CoreHandle, reset_stack, set_brightness};
use streamduck_core::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use streamduck_core::util::make_panel_unique;
use crate::daemon_data::{DaemonListener, DaemonRequest};
use std::io::Write;

/// Request for reloading all device configs
#[derive(Serialize, Deserialize)]
pub enum ReloadDeviceConfigsResult {
    /// Sent if error happened while reloading configs
    ConfigError,

    /// Sent if successfully reloaded configs
    Reloaded,
}

impl SocketData for ReloadDeviceConfigsResult {
    const NAME: &'static str = "reload_device_configs";
}

impl DaemonRequest for ReloadDeviceConfigsResult {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<ReloadDeviceConfigsResult>(packet) {
            match listener.config.reload_device_configs() {
                Ok(_) => {
                    for (serial, device) in listener.core_manager.list_added_devices() {
                        if !device.core.is_closed() {
                            if let Some(dvc_cfg) = listener.config.get_device_config(&serial) {
                                let handle = dvc_cfg.read().unwrap();
                                let wrapped_core = CoreHandle::wrap(device.core);

                                reset_stack(&wrapped_core, make_panel_unique(handle.layout.clone()));
                            }
                        }
                    }

                    send_packet(handle, packet, &ReloadDeviceConfigsResult::Reloaded).ok();
                },
                Err(err) => {
                    log::error!("Error encountered while reloading configs: {:?}", err);
                    send_packet(handle, packet, &ReloadDeviceConfigsResult::ConfigError).ok();
                }
            };
        }
    }
}

/// Request for reloading device config for specific device
#[derive(Serialize, Deserialize)]
pub struct ReloadDeviceConfig {
    pub serial_number: String
}

/// Response of [ReloadDeviceConfig] request
#[derive(Serialize, Deserialize)]
pub enum ReloadDeviceConfigResult {
    /// Sent if error happened while reloading configs
    ConfigError,

    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully reloaded configs
    Reloaded,
}

impl SocketData for ReloadDeviceConfig {
    const NAME: &'static str = "reload_device_config";
}

impl SocketData for ReloadDeviceConfigResult {
    const NAME: &'static str = "reload_device_config";
}

impl DaemonRequest for ReloadDeviceConfig {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ReloadDeviceConfig>(packet) {
            match listener.config.reload_device_config(&request.serial_number) {
                Ok(_) => {
                    if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                        if !device.core.is_closed() {
                            if let Some(dvc_cfg) = listener.config.get_device_config(&request.serial_number) {
                                let handle = dvc_cfg.read().unwrap();
                                let wrapped_core = CoreHandle::wrap(device.core);

                                reset_stack(&wrapped_core, make_panel_unique(handle.layout.clone()));
                            }
                        }
                    }

                    send_packet(handle, packet, &ReloadDeviceConfigResult::Reloaded).ok();
                },
                Err(err) => {
                    if let ConfigError::DeviceNotFound = err {
                        send_packet(handle, packet, &ReloadDeviceConfigResult::DeviceNotFound).ok();
                    } else {
                        log::error!("Error encountered while reloading config for {}: {:?}", request.serial_number, err);
                        send_packet(handle, packet, &ReloadDeviceConfigResult::ConfigError).ok();
                    }
                }
            }
        }
    }
}

/// Request for saving all device configs
#[derive(Serialize, Deserialize)]
pub enum SaveDeviceConfigsResult {
    /// Sent if error happened while saving configs
    ConfigError,

    /// Sent if successfully saved all configs
    Saved,
}

impl SocketData for SaveDeviceConfigsResult {
    const NAME: &'static str = "save_device_configs";
}

impl DaemonRequest for SaveDeviceConfigsResult {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if check_packet_for_data::<SaveDeviceConfigsResult>(packet) {
            match listener.config.save_device_configs() {
                Ok(_) => {
                    send_packet(handle, packet, &SaveDeviceConfigsResult::Saved).ok();
                },
                Err(err) => {
                    log::error!("Error encountered while saving configs: {:?}", err);
                    send_packet(handle, packet, &SaveDeviceConfigsResult::ConfigError).ok();
                }
            };
        }
    }
}

/// Request for saving device config for specific device
#[derive(Serialize, Deserialize)]
pub struct SaveDeviceConfig {
    pub serial_number: String,
}

/// Response of [SaveDeviceConfig] request
#[derive(Serialize, Deserialize)]
pub enum SaveDeviceConfigResult {
    /// Sent if error happened while saving config
    ConfigError,

    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully saved
    Saved,
}

impl SocketData for SaveDeviceConfig {
    const NAME: &'static str = "save_device_config";
}

impl SocketData for SaveDeviceConfigResult {
    const NAME: &'static str = "save_device_config";
}

impl DaemonRequest for SaveDeviceConfig {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SaveDeviceConfig>(packet) {
            match listener.config.save_device_config(&request.serial_number) {
                Ok(_) => {
                    send_packet(handle, packet, &SaveDeviceConfigResult::Saved).ok();
                },
                Err(err) => {
                    if let ConfigError::DeviceNotFound = err {
                        send_packet(handle, packet, &SaveDeviceConfigResult::DeviceNotFound).ok();
                    } else {
                        log::error!("Error encountered while saving config for {}: {:?}", request.serial_number, err);
                        send_packet(handle, packet, &SaveDeviceConfigResult::ConfigError).ok();
                    }
                }
            }
        }
    }
}

/// Request for exporting device config for specific device
#[derive(Serialize, Deserialize)]
pub struct GetDeviceConfig {
    pub serial_number: String,
}

/// Response of [GetDeviceConfig] request
#[derive(Serialize, Deserialize)]
pub enum GetDeviceConfigResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully exported
    Config(DeviceConfig),
}

impl SocketData for GetDeviceConfig {
    const NAME: &'static str = "get_device_config";
}

impl SocketData for GetDeviceConfigResult {
    const NAME: &'static str = "get_device_config";
}

impl DaemonRequest for GetDeviceConfig {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetDeviceConfig>(packet) {
            if let Some(config) = listener.config.get_device_config(&request.serial_number) {
                let config_handle = config.read().unwrap();
                send_packet(handle, packet, &GetDeviceConfigResult::Config(config_handle.clone())).ok();
            } else {
                send_packet(handle, packet, &GetDeviceConfigResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for exporting device config for specific device
#[derive(Serialize, Deserialize)]
pub struct ExportDeviceConfig {
    pub serial_number: String,
}

/// Response of [ExportDeviceConfig] request
#[derive(Serialize, Deserialize)]
pub enum ExportDeviceConfigResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if error happened during compression
    FailedToCompress,

    /// Sent if successfully exported
    Exported(String),
}

impl SocketData for ExportDeviceConfig {
    const NAME: &'static str = "export_device_config";
}

impl SocketData for ExportDeviceConfigResult {
    const NAME: &'static str = "export_device_config";
}

impl DaemonRequest for ExportDeviceConfig {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ExportDeviceConfig>(packet) {
            if let Some(config) = listener.config.get_device_config(&request.serial_number) {
                let config_handle = config.read().unwrap();
                let config = serde_json::to_string(config_handle.deref()).unwrap();

                // Compressing data
                let mut encoder = GzEncoder::new(vec![], Compression::default());
                write!(encoder, "{}", config).ok();

                if let Ok(byte_array) = encoder.finish() {
                    send_packet(handle, packet, &ExportDeviceConfigResult::Exported(base64::encode(byte_array))).ok();
                } else {
                    send_packet(handle, packet, &ExportDeviceConfigResult::FailedToCompress).ok();
                }
            } else {
                send_packet(handle, packet, &ExportDeviceConfigResult::DeviceNotFound).ok();
            }
        }
    }
}

/// Request for saving device config for specific device
#[derive(Serialize, Deserialize)]
pub struct ImportDeviceConfig {
    pub serial_number: String,
    pub config: String,
}

/// Response of [ImportDeviceConfig] request
#[derive(Serialize, Deserialize)]
pub enum ImportDeviceConfigResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if config was invalid
    InvalidConfig,

    /// Sent if config failed to save
    FailedToSave,

    /// Sent if successfully imported
    Imported,
}

impl SocketData for ImportDeviceConfig {
    const NAME: &'static str = "import_device_config";
}

impl SocketData for ImportDeviceConfigResult {
    const NAME: &'static str = "import_device_config";
}

impl DaemonRequest for ImportDeviceConfig {
    fn process(listener: &DaemonListener, handle: SocketHandle, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ImportDeviceConfig>(packet) {
            if let Ok(byte_array) = base64::decode(&request.config) {
                let mut decoder = GzDecoder::new(&byte_array[..]);
                let mut config = String::new();

                if let Ok(_) = decoder.read_to_string(&mut config) {
                    if let Ok(mut config) = serde_json::from_str::<DeviceConfig>(&config) {
                        if let Some(device) = listener.core_manager.get_device(&request.serial_number) {
                            config.serial = device.serial.clone();
                            config.vid = device.vid;
                            config.pid = device.pid;

                            listener.config.set_device_config(&request.serial_number, config.clone());

                            match listener.config.save_device_config(&request.serial_number) {
                                Ok(_) => {
                                    let wrapped_core = CoreHandle::wrap(device.core);

                                    reset_stack(&wrapped_core, make_panel_unique(config.layout));
                                    set_brightness(&wrapped_core, config.brightness);

                                    send_packet(handle, packet, &ImportDeviceConfigResult::Imported).ok();
                                }

                                Err(err) => {
                                    match err {
                                        ConfigError::IoError(_) | ConfigError::ParseError(_) => {
                                            send_packet(handle, packet, &ImportDeviceConfigResult::FailedToSave).ok();
                                        }

                                        ConfigError::DeviceNotFound => {
                                            send_packet(handle, packet, &ImportDeviceConfigResult::DeviceNotFound).ok();
                                        }
                                    }
                                }
                            }
                        } else {
                            send_packet(handle, packet, &ImportDeviceConfigResult::DeviceNotFound).ok();
                        }
                    } else {
                        send_packet(handle, packet, &ImportDeviceConfigResult::InvalidConfig).ok();
                    }
                } else {
                    send_packet(handle, packet, &ImportDeviceConfigResult::InvalidConfig).ok();
                }
            } else {
                send_packet(handle, packet, &ImportDeviceConfigResult::InvalidConfig).ok();
            }
        }
    }
}