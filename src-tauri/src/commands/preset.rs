use crate::preset::builtin::{self, Preset};
use crate::preset::templates::{self, Template};

#[tauri::command]
pub fn list_presets() -> Vec<Preset> {
    builtin::builtin_presets()
}

#[tauri::command]
pub fn list_templates() -> Vec<Template> {
    templates::builtin_templates()
}
