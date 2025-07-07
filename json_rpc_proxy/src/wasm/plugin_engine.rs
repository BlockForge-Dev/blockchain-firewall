// src/wasm/plugin_engine.rs

use std::{fs, path::Path};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::WasiCtx;
use tracing::error;

pub struct WasmPlugin {
    engine: Engine,
    linker: Linker<WasiCtx>,
    store: Store<WasiCtx>,
    instance: Instance,
    memory: Memory,
    allow_func: TypedFunc<(i32, i32), i32>,
}

impl WasmPlugin {
    pub fn load(path: &str) -> Result<Self, anyhow::Error> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker(&mut linker, |ctx| ctx)?;

        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
        let mut store = Store::new(&engine, wasi_ctx);

        let module_bytes = fs::read(path)?;
        let module = Module::new(&engine, &module_bytes)?;

        let instance = linker.instantiate(&mut store, &module)?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("failed to find `memory` export"))?;

        let allow_func = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "should_allow")?;

        Ok(Self {
            engine,
            linker,
            store,
            instance,
            memory,
            allow_func,
        })
    }

    pub fn should_allow(&mut self, method: &str) -> bool {
        let bytes = method.as_bytes();
        let len = bytes.len() as i32;
        let ptr = 1000; // simple static offset

        // Write the method name to WASM memory
        if let Err(e) = self.memory.write(&mut self.store, ptr as usize, bytes) {
            error!("Failed to write to WASM memory: {:?}", e);
            return false;
        }

        // Call the WASM function with (ptr, len)
        match self.allow_func.call(&mut self.store, (ptr, len)) {
            Ok(result) => result == 1,
            Err(err) => {
                error!("Plugin error: {:?}", err);
                false
            }
        }
    }
}
