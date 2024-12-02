#![allow(non_snake_case)]

use renderer_core::game::TextureId;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmModelInfo {
    pub id: String,
    pub transform: WasmTransform,
    pub material_info: WasmMaterialInfo,
    pub shader_id: String,
    pub instance_count: u32,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: f32,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmFrameTime {
    pub avg_delta_time: f32,
    pub avg_gpu_time: f32,
}

impl From<WasmTransform> for renderer_core::transform::Transform {
    fn from(v: WasmTransform) -> Self {
        renderer_core::transform::Transform {
            position: v.position.into(),
            rotation: glam::Quat::from_euler(
                glam::EulerRot::XYZ,
                v.rotation[0],
                v.rotation[1],
                v.rotation[2],
            ),
            scale: v.scale,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmMaterialInfo {
    pub color: [f32; 3],
    pub emissive: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub diffuse_texture: Option<String>,
}

impl From<WasmMaterialInfo> for renderer_core::game::MaterialInfo {
    fn from(v: WasmMaterialInfo) -> Self {
        renderer_core::game::MaterialInfo {
            color: v.color.into(),
            emissive: v.emissive.into(),
            roughness: v.roughness,
            metallic: v.metallic,
            diffuse_texture: v.diffuse_texture.map(TextureId),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmShaderInfo {
    pub id: String,
    pub label: String,
    pub code: String,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmCompilationMessage {
    pub message: String,
    pub message_type: WasmCompilationMessageType,
    pub location: Option<WasmSourceLocation>,
}

impl From<wgpu::CompilationMessage> for WasmCompilationMessage {
    fn from(v: wgpu::CompilationMessage) -> Self {
        Self {
            message: v.message,
            message_type: v.message_type.into(),
            location: v.location.map(WasmSourceLocation::from),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmCompilationMessageType {
    Error,
    Warning,
    Info,
}

impl From<wgpu::CompilationMessageType> for WasmCompilationMessageType {
    fn from(v: wgpu::CompilationMessageType) -> Self {
        match v {
            wgpu::CompilationMessageType::Error => Self::Error,
            wgpu::CompilationMessageType::Warning => Self::Warning,
            wgpu::CompilationMessageType::Info => Self::Info,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmSourceLocation {
    /// 1-based line number.
    pub line_number: u32,
    /// 1-based column in code units (in bytes) of the start of the span.
    /// Remember to convert accordingly when displaying to the user.
    pub line_position: u32,
    /// 0-based Offset in code units (in bytes) of the start of the span.
    pub offset: u32,
    /// Length in code units (in bytes) of the span.
    pub length: u32,
}

impl From<wgpu::SourceLocation> for WasmSourceLocation {
    fn from(v: wgpu::SourceLocation) -> Self {
        Self {
            line_number: v.line_number,
            line_position: v.line_position,
            offset: v.offset,
            length: v.length,
        }
    }
}

/// Can store a https://developer.mozilla.org/en-US/docs/Web/API/ImageData
#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmTexture {
    pub id: String,
    pub width: u32,
    /// RGBA
    pub data: Vec<u8>,
}
