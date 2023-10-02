use std::fs;
use std::path::PathBuf;

use crate::Theme;

pub fn apply_theme(theme: &Theme) -> Result<(), &'static str> {
    let asar_path = discord_asar_path()?;
    let backup_path = asar_path.with_file_name("app.asar.bak");

    if !asar_path.exists() {
        return Err("Could not find Discord app.asar");
    }

    if !backup_path.exists() {
        fs::copy(&asar_path, &backup_path)
            .map_err(|_| "Failed to backup app.asar to app.asar.bak")?;
    }

    let asar_content = fs::read(&backup_path).map_err(|_| "Failed to read app.asar file.\nPlease make sure Discord is closed before applying the theme.")?;
    let asar_reader = asar::AsarReader::new(&asar_content, backup_path)
        .map_err(|_| "Failed to parse app.asar file")?;

    let mut asar_writer = asar::AsarWriter::new();
    for (path, file) in asar_reader.files() {
        if *path == PathBuf::from("app_bootstrap/bootstrap.js") {
            let injected_css = format!(
                r#"
                const CSS = `
                    * {{
                        --background-primary: {} !important;
                        --background-secondary: {} !important;
                        --background-secondary-alt: {} !important;
                        --background-tertiary: {} !important;
                        --text-normal: {} !important;
                        --text-muted: {} !important;
                        --text-link: {} !important;
                        --header-primary: {} !important;
                        --header-secondary: {} !important;
                        --channels-default: {} !important;
                    }}
                `;
            "#,
                theme.background_primary.web_css(),
                theme.background_secondary.web_css(),
                theme.background_secondary_alt.web_css(),
                theme.background_tertiary.web_css(),
                theme.text_normal.web_css(),
                theme.text_muted.web_css(),
                theme.text_hyperlink.web_css(),
                theme.header_primary.web_css(),
                theme.header_secondary.web_css(),
                theme.channels_default.web_css()
            );
            let loader_js = r#"
            app.on('browser-window-created', (_event, window) => {
                window.webContents.on('did-finish-load', () => {
                    window.webContents.insertCSS(CSS);
                    console.log('Injected Discord Theme CSS');
                });
            });
            "#;
            asar_writer
                .write_file(
                    path,
                    [file.data(), injected_css.as_bytes(), loader_js.as_bytes()].concat(),
                    false,
                )
                .map_err(|_| "Failed to write modified bootstrap.js file to asar")?;
        } else {
            asar_writer
                .write_file(path, file.data(), false)
                .map_err(|_| "Failed to add file to new app.asar")?;
        }
    }

    let asar_file = fs::File::create(asar_path)
        .map_err(|_| "Failed to open app.asar for writing.\nPlease make sure Discord is closed before applying the theme.")?;
    asar_writer.finalize(asar_file)
        .map_err(|_| "Failed to write content to app.asar.\nPlease make sure Discord is closed before applying the theme.")?;

    Ok(())
}

pub fn reset_theme() -> Result<(), &'static str> {
    let asar_path = discord_asar_path()?;
    let backup_path = asar_path.with_file_name("app.asar.bak");

    if backup_path.exists() {
        let _ = fs::remove_file(&asar_path);
        fs::copy(backup_path, asar_path)
            .map_err(|_| "Failed to restore app.asar.bak to app.asar")?;
        Ok(())
    } else {
        Err("Failed to find app.asar.bak backup file")
    }
}

#[cfg(target_os = "linux")]
fn discord_asar_path() -> Result<PathBuf, &'static str> {
    let mut path = which::which("discord")
        .map_err(|_| "Could not find discord binary on PATH")?
        .canonicalize()
        .map_err(|_| "Could not canonicalise path of discord binary")?
        .parent()
        .ok_or("ASDF")?
        .to_path_buf();
    path.push("resources");
    path.push("app.asar");
    Ok(path)
}

#[cfg(target_os = "windows")]
fn discord_asar_path() -> Result<PathBuf, &'static str> {
    let mut path = PathBuf::from(
        std::env::var("LOCALAPPDATA")
            .map_err(|_| "Failed to get LOCALAPPDATA path environment variable")?,
    );
    path.push("Discord");
    for entry in fs::read_dir(path)
        .map_err(|_| "Failed to read discord app directory")?
        .flatten()
    {
        if entry.file_name().to_string_lossy().starts_with("app-") {
            let mut path = entry.path();
            path.push("resources");
            path.push("app.asar");
            return Ok(path);
        }
    }
    Err("Failed to find Discord app directory")
}

#[cfg(target_os = "macos")]
fn discord_asar_path() -> Result<PathBuf, &'static str> {
    Ok("/Applications/Discord.app/Contents/Resources/app.asar".into())
}
