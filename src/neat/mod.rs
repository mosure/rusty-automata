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


const NEAT_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 21533341678341);

#[derive(Default)]
pub struct NeatPlugin;

impl Plugin for NeatPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            NEAT_SHADER_HANDLE,
            "neat.wgsl",
            Shader::from_wgsl
        );
    }
}
