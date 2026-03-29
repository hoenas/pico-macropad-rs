use alloc::string::String;
use embedded_sdmmc::{BlockDevice, Directory, Mode, ShortFileName, TimeSource};

use crate::{ButtonConfig, KeyboardCode, MacroConfig, RotaryEncoderConfig};

const LAST_CONFIG_FILE_NAME: &str = "lastcfg";
const FILE_READ_BUFFER_SIZE: usize = 1024;

pub fn get_last_config(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
) -> String {
    let mut buffer: [u8; 12] = [0; 12];
    let last_config_file_name_handle = root_dir
        .open_file_in_dir(LAST_CONFIG_FILE_NAME, Mode::ReadOnly)
        .unwrap();
    let length = last_config_file_name_handle.read(&mut buffer).unwrap();
    String::from(core::str::from_utf8(&buffer[..length]).unwrap())
}

pub fn read_config_file(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    filename: &impl AsRef<str>,
) -> MacroConfig {
    // Load config file
    let short_file_name = ShortFileName::create_from_str(filename.as_ref()).unwrap();
    let opened_file = root_dir
        .open_file_in_dir(short_file_name, embedded_sdmmc::Mode::ReadOnly)
        .unwrap();
    let mut buffer = [0u8; FILE_READ_BUFFER_SIZE];
    let bytes_read = opened_file.read(&mut buffer).unwrap();
    let config: MacroConfig = serde_json::from_slice(&buffer[..bytes_read]).unwrap();
    config
}

pub fn write_last_config(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    filename: &impl AsRef<str>,
) {
    let short_file_name = ShortFileName::create_from_str(LAST_CONFIG_FILE_NAME).unwrap();
    let opened_file = root_dir
        .open_file_in_dir(
            short_file_name,
            embedded_sdmmc::Mode::ReadWriteCreateOrTruncate,
        )
        .unwrap();
    opened_file.write(filename.as_ref().as_bytes()).unwrap();
}

pub fn write_example_config_file(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    filename: &impl AsRef<str>,
) {
    let example_config = crate::example_config::get_example_config();
    let short_file_name = ShortFileName::create_from_str(filename.as_ref()).unwrap();
    let opened_file = root_dir
        .open_file_in_dir(
            short_file_name,
            embedded_sdmmc::Mode::ReadWriteCreateOrTruncate,
        )
        .unwrap();
    let config_json = serde_json::to_string(&example_config).unwrap();
    opened_file.write(config_json.as_bytes()).unwrap();
}
