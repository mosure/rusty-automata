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


const NOISE_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 8343684782112);

#[derive(Default)]
pub struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            NOISE_SHADER_HANDLE,
            "noise.wgsl",
            Shader::from_wgsl
        );
    }
}
