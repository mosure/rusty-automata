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


const AUTOMATA_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6712956732940);

#[derive(Default)]
pub struct AutomataPlugin;

impl Plugin for AutomataPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            AUTOMATA_SHADER_HANDLE,
            "automata.wgsl",
            Shader::from_wgsl
        );
    }
}
