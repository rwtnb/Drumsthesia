use std::{path::PathBuf};

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(h) })
        .map(PathBuf::from)
}

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
fn xdg_config() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(h) })
        .map(PathBuf::from)
        .map(|p| p.join("neothesia"))
        .or_else(|| home().map(|h| h.join(".config").join("neothesia")))
}

pub fn default_sf2() -> Option<PathBuf> {
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    {
        if let Some(path) = xdg_config().map(|p| p.join("Arachno SoundFont - Version 1.0.sf2")) {
            if path.exists() {
                return Some(path);
            }
        }

        let flatpak = PathBuf::from("/app/share/drumsthesia/Arachno SoundFont - Version 1.0.sf2");
        if flatpak.exists() {
            Some(flatpak)
        } else {
            None
        }
    }

    #[cfg(target_os = "windows")]
    return Some(PathBuf::from("./Arachno SoundFont - Version 1.0.sf2"));

    #[cfg(target_os = "macos")]
    return bundled_resource_path("Arachno SoundFont - Version 1.0", "sf2").map(PathBuf::from);
}

pub fn settings_ron() -> Option<PathBuf> {
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    return xdg_config().map(|p| p.join("settings.ron"));

    #[cfg(target_os = "windows")]
    return Some(PathBuf::from("./settings.ron"));

    #[cfg(target_os = "macos")]
    return bundled_resource_path("settings", "ron").map(PathBuf::from);
}

#[cfg(target_os = "macos")]
fn bundled_resource_path(name: &str, extension: &str) -> Option<String> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let cls = Class::get("NSBundle").unwrap();
        let bundle: *mut Object = msg_send![cls, mainBundle];
        let cls = Class::get("NSString").unwrap();
        let objc_str: *mut Object = msg_send![cls, alloc];
        let objc_name: *mut Object = msg_send![objc_str,
                                              initWithBytes:name.as_ptr()
                                              length:name.len()
                                              encoding: 4]; // UTF8_ENCODING
        let objc_str: *mut Object = msg_send![cls, alloc];
        let objc_ext: *mut Object = msg_send![objc_str,
                                              initWithBytes:extension.as_ptr()
                                              length:extension.len()
                                              encoding: 4]; // UTF8_ENCODING
        let ini: *mut Object = msg_send![bundle,
                                         pathForResource:objc_name
                                         ofType:objc_ext];
        let _: () = msg_send![objc_name, release];
        let _: () = msg_send![objc_ext, release];
        let cstr: *const i8 = msg_send![ini, UTF8String];
        if !cstr.is_null() {
            let rstr = std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned();
            return Some(rstr);
        }
        None
    }
}
