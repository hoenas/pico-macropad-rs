use alloc::string::{String, ToString};
use anyhow::Error;
use embedded_sdmmc::{BlockDevice, Directory, ShortFileName, TimeSource};

use crate::MacroConfig;

const LAST_CONFIG_FILE_NAME: &str = "lastcfg";
const FILE_READ_BUFFER_SIZE: usize = 8196;

pub fn read_file(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    filename: &str,
) -> anyhow::Result<(usize, [u8; FILE_READ_BUFFER_SIZE])> {
    let short_file_name = ShortFileName::create_from_str(filename)
        .map_err(|_| Error::msg("Failed to create short file name"))?;
    let opened_file = root_dir
        .open_file_in_dir(short_file_name, embedded_sdmmc::Mode::ReadOnly)
        .map_err(|_| Error::msg("Failed to open file"))?;
    let mut buffer = [0u8; FILE_READ_BUFFER_SIZE];
    let bytes_read = opened_file
        .read(&mut buffer)
        .map_err(|_| Error::msg("Failed to read file"))?;
    Ok((bytes_read, buffer))
}

pub fn get_last_config(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
) -> anyhow::Result<String> {
    let (bytes_read, buffer) = read_file(root_dir, LAST_CONFIG_FILE_NAME)?;
    let file_name = core::str::from_utf8(&buffer[..bytes_read])?;
    Ok(file_name.to_string())
}

pub fn read_config_file(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    filename: &str,
) -> anyhow::Result<MacroConfig> {
    // Load config file
    let (bytes_read, buffer) = read_file(root_dir, filename)?;
    let mut config: MacroConfig = serde_cbor::from_slice(&buffer[..bytes_read])
        .map_err(|_| Error::msg("Failed to parse config"))?;
    // Reverse keystrokes for easier popping later
    config.buttons[0].keystroke.reverse();
    config.buttons[1].keystroke.reverse();
    config.buttons[2].keystroke.reverse();
    config.buttons[3].keystroke.reverse();
    config.buttons[4].keystroke.reverse();
    config.buttons[5].keystroke.reverse();
    config.buttons[6].keystroke.reverse();
    config.buttons[7].keystroke.reverse();
    config.buttons[8].keystroke.reverse();
    config.buttons[9].keystroke.reverse();
    config.menu_encoder.keystroke_left.reverse();
    config.menu_encoder.keystroke_right.reverse();
    config.encoders[0].keystroke_left.reverse();
    config.encoders[0].keystroke_right.reverse();
    config.encoders[0].keystroke_push.reverse();
    config.encoders[1].keystroke_left.reverse();
    config.encoders[1].keystroke_right.reverse();
    config.encoders[1].keystroke_push.reverse();
    Ok(config)
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
    let config_cbor = serde_cbor::to_vec(&example_config).unwrap();
    opened_file.write(&config_cbor).unwrap();
}
