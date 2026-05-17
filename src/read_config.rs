use alloc::{string::String, vec::Vec};
use anyhow::Error;
use embedded_sdmmc::{BlockDevice, Directory, ShortFileName, TimeSource};
use embedded_graphics::geometry::Point;
use tinybmp::RawBmp;

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
    let mut config: MacroConfig = serde_json::from_slice(&buffer[..bytes_read])
        .map_err(|_| Error::msg("Failed to parse config"))?;
    load_icon_files(root_dir, &mut config)?;
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
    let config_json = serde_json::to_string(&example_config).unwrap();
    opened_file.write(config_json.as_bytes()).unwrap();
}

fn normalize_icon_path(path: &str) -> Option<&str> {
    let trimmed = path.trim().trim_start_matches('/').trim_start_matches('\\');
    if trimmed.is_empty() {
        return None;
    }
    let normalized = if let Some(stripped) = trimmed.strip_prefix("icons/") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("icons\\") {
        stripped
    } else {
        trimmed
    };
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn read_icon_file(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    path: &str,
) -> anyhow::Result<Option<Vec<u8>>> {
    let normalized = match normalize_icon_path(path) {
        Some(path) => path,
        None => return Ok(None),
    };
    let path = normalized.replace('\\', "/");
    let mut segments = path.split('/').filter(|s| !s.is_empty());
    let file_name = match segments.next_back() {
        Some(name) => name,
        None => return Ok(None),
    };
    let mut dir = root_dir.open_dir("icons").map_err(|_| Error::msg("Failed to open icons directory"))?;
    for segment in segments {
        dir.change_dir(segment)
            .map_err(|_| Error::msg("Failed to open icon subdirectory"))?;
    }
    let short_file_name = ShortFileName::create_from_str(file_name)
        .map_err(|_| Error::msg("Failed to create short file name for icon"))?;
    let opened_file = dir
        .open_file_in_dir(short_file_name, embedded_sdmmc::Mode::ReadOnly)
        .map_err(|_| Error::msg("Failed to open icon file"))?;
    let mut buffer = [0u8; FILE_READ_BUFFER_SIZE];
    let bytes_read = opened_file
        .read(&mut buffer)
        .map_err(|_| Error::msg("Failed to read icon file"))?;
    let bmp = RawBmp::from_slice(&buffer[..bytes_read]).map_err(|_| Error::msg("Failed to parse BMP icon"))?;
    if bmp.header().image_size.width != 20 || bmp.header().image_size.height != 20 {
        return Err(Error::msg("Icon BMP must be 20x20 pixels"));
    }
    let mut pixels = Vec::new();
    pixels.reserve(400);
    for y in 0..20 {
        for x in 0..20 {
            let value = bmp
                .pixel(Point::new(x, y))
                .ok_or_else(|| Error::msg("Failed to read BMP pixel"))?;
            pixels.push(if value != 0 { 1 } else { 0 });
        }
    }
    Ok(Some(pixels))
}

fn load_icon_files(
    root_dir: &Directory<'_, impl BlockDevice, impl TimeSource, 4, 4, 1>,
    config: &mut MacroConfig,
) -> anyhow::Result<()> {
    config.button0.display_icon_pixels = read_icon_file(root_dir, config.button0.display_icon_path.as_deref().unwrap_or(""))?;
    config.button1.display_icon_pixels = read_icon_file(root_dir, config.button1.display_icon_path.as_deref().unwrap_or(""))?;
    config.button2.display_icon_pixels = read_icon_file(root_dir, config.button2.display_icon_path.as_deref().unwrap_or(""))?;
    config.button3.display_icon_pixels = read_icon_file(root_dir, config.button3.display_icon_path.as_deref().unwrap_or(""))?;
    config.button4.display_icon_pixels = read_icon_file(root_dir, config.button4.display_icon_path.as_deref().unwrap_or(""))?;
    config.button5.display_icon_pixels = read_icon_file(root_dir, config.button5.display_icon_path.as_deref().unwrap_or(""))?;
    config.button6.display_icon_pixels = read_icon_file(root_dir, config.button6.display_icon_path.as_deref().unwrap_or(""))?;
    config.button7.display_icon_pixels = read_icon_file(root_dir, config.button7.display_icon_path.as_deref().unwrap_or(""))?;
    config.button8.display_icon_pixels = read_icon_file(root_dir, config.button8.display_icon_path.as_deref().unwrap_or(""))?;
    config.button9.display_icon_pixels = read_icon_file(root_dir, config.button9.display_icon_path.as_deref().unwrap_or(""))?;
    config.menu_encoder.display_icon_pixels = read_icon_file(root_dir, config.menu_encoder.display_icon_path.as_deref().unwrap_or(""))?;
    config.encoder1.display_icon_pixels = read_icon_file(root_dir, config.encoder1.display_icon_path.as_deref().unwrap_or(""))?;
    config.encoder2.display_icon_pixels = read_icon_file(root_dir, config.encoder2.display_icon_path.as_deref().unwrap_or(""))?;
    Ok(())
}
