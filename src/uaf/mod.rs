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


const UAF_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 61270573934);

#[derive(Default)]
pub struct UafPlugin;

impl Plugin for UafPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            UAF_SHADER_HANDLE,
            "uaf.wgsl",
            Shader::from_wgsl
        );
    }
}
