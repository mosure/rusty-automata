use bevy::{
    asset::{
        load_internal_asset,
        HandleUntyped,
    },
    prelude::*,
    reflect::{
        TypeUuid,
    }
};


const PLOT_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 809823407934);

#[derive(Default)]
pub struct PlotPlugin;

impl Plugin for PlotPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PLOT_SHADER_HANDLE,
            "plot.wgsl",
            Shader::from_wgsl
        );
    }
}
