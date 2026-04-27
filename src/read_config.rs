use alloc::{string::String, vec::Vec};
use anyhow::Error;
use embedded_sdmmc::{BlockDevice, Directory, ShortFileName, TimeSource};

use crate::MacroConfig;

const LAST_CONFIG_FILE_NAME: &str = "lastcfg";
const ICON_DIRECTORY: &str = "icons";
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
    let mut config: MacroConfig = serde_json::from_slice(&buffer[..bytes_read])
        .map_err(|_| Error::msg("Failed to parse config"))?;
    fill_icon_pixels(root_dir, &mut config);
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

fn fill_icon_pixels(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    config: &mut MacroConfig,
) {
    config.button0.display_icon_pixels =
        load_icon_pixels(root_dir, config.button0.display_icon.as_deref());
    config.button1.display_icon_pixels =
        load_icon_pixels(root_dir, config.button1.display_icon.as_deref());
    config.button2.display_icon_pixels =
        load_icon_pixels(root_dir, config.button2.display_icon.as_deref());
    config.button3.display_icon_pixels =
        load_icon_pixels(root_dir, config.button3.display_icon.as_deref());
    config.button4.display_icon_pixels =
        load_icon_pixels(root_dir, config.button4.display_icon.as_deref());
    config.button5.display_icon_pixels =
        load_icon_pixels(root_dir, config.button5.display_icon.as_deref());
    config.button6.display_icon_pixels =
        load_icon_pixels(root_dir, config.button6.display_icon.as_deref());
    config.button7.display_icon_pixels =
        load_icon_pixels(root_dir, config.button7.display_icon.as_deref());
    config.button8.display_icon_pixels =
        load_icon_pixels(root_dir, config.button8.display_icon.as_deref());
    config.button9.display_icon_pixels =
        load_icon_pixels(root_dir, config.button9.display_icon.as_deref());
    config.menu_encoder.display_icon_pixels =
        load_icon_pixels(root_dir, config.menu_encoder.display_icon.as_deref());
    config.encoder1.display_icon_pixels =
        load_icon_pixels(root_dir, config.encoder1.display_icon.as_deref());
    config.encoder2.display_icon_pixels =
        load_icon_pixels(root_dir, config.encoder2.display_icon.as_deref());
}

fn load_icon_pixels(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    icon_name: Option<&str>,
) -> Option<Vec<u8>> {
    icon_name
        .and_then(|name| read_icon(root_dir, name).ok())
        .and_then(|data| parse_bmp_icon(&data).ok())
}

fn parse_bmp_icon(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    if data.len() < 54 || &data[0..2] != b"BM" {
        return Err(Error::msg("Invalid BMP icon file"));
    }
    let pixel_offset = u32::from_le_bytes(data[10..14].try_into().unwrap()) as usize;
    let width = i32::from_le_bytes(data[18..22].try_into().unwrap());
    let height = i32::from_le_bytes(data[22..26].try_into().unwrap());
    let planes = u16::from_le_bytes(data[26..28].try_into().unwrap());
    let bit_count = u16::from_le_bytes(data[28..30].try_into().unwrap());
    if planes != 1 || bit_count != 1 {
        return Err(Error::msg("Unsupported BMP bit depth"));
    }
    let width = width as usize;
    let height = height.abs() as usize;
    if width != 20 || height != 20 {
        return Err(Error::msg("Icon BMP must be 20x20"));
    }
    let row_bytes = ((width + 31) / 32) * 4;
    if data.len() < pixel_offset + row_bytes * height {
        return Err(Error::msg("BMP icon data truncated"));
    }
    let top_down = i32::from_le_bytes(data[22..26].try_into().unwrap()) < 0;
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        let row = if top_down { y } else { height - 1 - y };
        let row_start = pixel_offset + row * row_bytes;
        for x in 0..width {
            let byte = data[row_start + x / 8];
            let bit = 7 - (x % 8);
            pixels.push(if (byte >> bit) & 1 != 0 { 1 } else { 0 });
        }
    }
    Ok(pixels)
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

pub fn read_icon(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    icon_name: &str,
) -> Result<alloc::vec::Vec<u8>, Error> {
    let icon_dir = root_dir
        .open_dir(ICON_DIRECTORY)
        .map_err(|_| Error::msg("Failed to open icon directory"))?;
    let (bytes_read, buffer) = read_file(&icon_dir, icon_name)?;
    Ok(buffer[..bytes_read].to_vec())
}
