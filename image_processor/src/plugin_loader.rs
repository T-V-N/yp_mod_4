use libloading::{Library, Symbol};
use std::os::raw::c_char;

#[cfg(target_os = "windows")]
const LIB_EXT: &str = "dll";

#[cfg(target_os = "macos")]
const LIB_EXT: &str = "dylib";

#[cfg(target_os = "linux")]
const LIB_EXT: &str = "so";

pub struct PluginInterface<'a> {
    pub process_image: Symbol<
        'a,
        unsafe extern "C" fn(height: u32, width: u32, rgba_data: *mut u8, params: *const c_char),
    >,
}

pub struct Plugin {
    plugin: Library,
}

impl Plugin {
    pub fn new(filename: &str) -> Result<Self, libloading::Error> {
        let path = format!("{}.{}", filename, LIB_EXT);
        Ok(Plugin {
            plugin: unsafe { Library::new(path) }?,
        })
    }
    pub fn interface(&self) -> Result<PluginInterface<'_>, libloading::Error> {
        Ok(PluginInterface {
            process_image: unsafe { self.plugin.get("process_image") }?,
        })
    }
}
