use alloc::string::String;
use anyhow::Error;
use embedded_sdmmc::{BlockDevice, Directory, ShortFileName, TimeSource};

use crate::MacroConfig;

const LAST_CONFIG_FILE_NAME: &str = "lastcfg";
const FILE_READ_BUFFER_SIZE: usize = 4096;

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
    Ok(String::from(core::str::from_utf8(&buffer[..bytes_read])?))
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
    config.button0.keystroke.reverse();
    config.button1.keystroke.reverse();
    config.button2.keystroke.reverse();
    config.button3.keystroke.reverse();
    config.button4.keystroke.reverse();
    config.button5.keystroke.reverse();
    config.button6.keystroke.reverse();
    config.button7.keystroke.reverse();
    config.button8.keystroke.reverse();
    config.button9.keystroke.reverse();
    config.menu_encoder.keystroke_left.reverse();
    config.menu_encoder.keystroke_right.reverse();
    config.encoder1.keystroke_left.reverse();
    config.encoder1.keystroke_right.reverse();
    config.encoder1.keystroke_push.reverse();
    config.encoder2.keystroke_left.reverse();
    config.encoder2.keystroke_right.reverse();
    config.encoder2.keystroke_push.reverse();
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
