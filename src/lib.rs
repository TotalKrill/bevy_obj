mod loader;
pub use loader::*;

use bevy::app::prelude::*;
use bevy::asset::AddAsset;

/// Adds support for Obj file loading to Apps
#[derive(Default)]
pub struct ObjPlugin;

impl Plugin for ObjPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<ObjLoader>();
    }
}
