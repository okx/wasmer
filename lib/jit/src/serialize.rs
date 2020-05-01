use crate::data::OwnedDataInitializer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasmer_compiler::{
    Compilation, CompiledFunctionFrameInfo, CompiledFunctionUnwindInfo, JumpTableOffsets,
    Relocation,
};
use wasmer_runtime::Module;

use wasm_common::entity::PrimaryMap;
use wasm_common::{LocalFuncIndex, MemoryIndex, TableIndex};
use wasmer_runtime::{MemoryPlan, TablePlan};

/// The function body.
/// 
/// Note: We separate it into it's own struct to make it serializable
/// with `serde_bytes`.s
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct FunctionBody {
    /// The function body.
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedCompilation {
    pub function_bodies: PrimaryMap<LocalFuncIndex, FunctionBody>,
    pub function_relocations: PrimaryMap<LocalFuncIndex, Vec<Relocation>>,
    pub function_jt_offsets: PrimaryMap<LocalFuncIndex, JumpTableOffsets>,
    pub function_unwind_info: PrimaryMap<LocalFuncIndex, CompiledFunctionUnwindInfo>,
    pub function_frame_info: PrimaryMap<LocalFuncIndex, CompiledFunctionFrameInfo>,
}

/// Structure to cache the content ot the compilation
#[derive(Serialize, Deserialize)]
pub struct SerializedModule {
    pub compilation: Compilation,
    // pub compilation: SerializedCompilation,
    pub module: Arc<Module>,
    pub data_initializers: Box<[OwnedDataInitializer]>,
    // Plans for that module
    pub memory_plans: PrimaryMap<MemoryIndex, MemoryPlan>,
    pub table_plans: PrimaryMap<TableIndex, TablePlan>,
}
