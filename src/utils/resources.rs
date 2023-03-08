use std::{env, path::PathBuf};

fn home() -> Option<PathBuf> {
    env::var_os("HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(h) })
        .map(PathBuf::from)
}

fn xdg_config() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(h) })
        .map(PathBuf::from)
        .map(|p| p.join("neothesia"))
        .or_else(|| home().map(|h| h.join(".config").join("neothesia")))
}

pub fn default_sf2() -> Option<PathBuf> {
    return Some(PathBuf::from("/Users/rodrigo/Downloads/TimGM6mb-MuseScore.sf2"));
}

pub fn settings_ron() -> Option<PathBuf> {
    return xdg_config().map(|p| p.join("settings.ron"));
}
