//! WebC container support for running Emscripten modules

use std::sync::Arc;

use anyhow::{anyhow, Context, Error};
use serde::{Deserialize, Serialize};
use wasmer::{FunctionEnv, Instance, Module, Store};
use wasmer_emscripten::{
    generate_emscripten_env, is_emscripten_module, run_emscripten_instance, EmEnv,
    EmscriptenGlobals,
};
use webc::{
    metadata::{annotations::Emscripten, Command},
    Container,
};

use crate::WasiRuntime;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EmscriptenRunner {
    args: Vec<String>,
}

impl EmscriptenRunner {
    /// Constructs a new `EmscriptenRunner` given an `Store`
    pub fn new() -> Self {
        EmscriptenRunner::default()
    }

    /// Returns the current arguments for this `EmscriptenRunner`
    pub fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }

    /// Builder method to provide CLI args to the runner
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.set_args(args);
        self
    }

    /// Set the CLI args
    pub fn set_args(&mut self, args: Vec<String>) {
        self.args = args;
    }
}

impl crate::runners::Runner for EmscriptenRunner {
    fn can_run_command(command: &Command) -> Result<bool, Error> {
        Ok(command
            .runner
            .starts_with(webc::metadata::annotations::EMSCRIPTEN_RUNNER_URI))
    }

    #[allow(unreachable_code, unused_variables)]
    fn run_command(
        &mut self,
        command_name: &str,
        container: &Container,
        runtime: Arc<dyn WasiRuntime + Send + Sync>,
    ) -> Result<(), Error> {
        let command = container
            .manifest()
            .commands
            .get(command_name)
            .context("Command not found")?;

        let Emscripten {
            atom: atom_name,
            main_args,
            ..
        } = command.annotation("emscripten")?.unwrap_or_default();
        let atom_name = atom_name.context("The atom name is required")?;
        let atoms = container.atoms();
        let atom_bytes = atoms
            .get(&atom_name)
            .with_context(|| format!("Unable to read the \"{atom_name}\" atom"))?;

        let mut module = crate::runtime::compile_module(atom_bytes, &*runtime)?;
        module.set_name(&atom_name);

        let mut store = runtime.new_store();
        let (mut globals, env) = prepare_emscripten_env(&mut store, &module, &atom_name)?;

        exec_module(
            &mut store,
            &module,
            &mut globals,
            env,
            &atom_name,
            main_args.unwrap_or_default(),
        )?;

        Ok(())
    }
}

fn prepare_emscripten_env(
    store: &mut Store,
    module: &Module,
    name: &str,
) -> Result<(EmscriptenGlobals, FunctionEnv<EmEnv>), anyhow::Error> {
    if !is_emscripten_module(module) {
        return Err(anyhow!("Atom {name:?} is not an emscripten module"));
    }

    let env = FunctionEnv::new(store, EmEnv::new());
    let emscripten_globals = EmscriptenGlobals::new(store, &env, module);
    let emscripten_globals = emscripten_globals.map_err(|e| anyhow!("{}", e))?;
    env.as_mut(store)
        .set_data(&emscripten_globals.data, Default::default());

    Ok((emscripten_globals, env))
}

fn exec_module(
    store: &mut Store,
    module: &Module,
    globals: &mut EmscriptenGlobals,
    em_env: FunctionEnv<EmEnv>,
    name: &str,
    args: Vec<String>,
) -> Result<(), anyhow::Error> {
    let import_object = generate_emscripten_env(store, &em_env, globals);

    let mut instance = Instance::new(store, module, &import_object)
        .map_err(|e| anyhow!("Cant instantiate emscripten module {name:?}: {e}"))?;

    run_emscripten_instance(
        &mut instance,
        em_env.into_mut(store),
        globals,
        name,
        args.iter().map(|arg| arg.as_str()).collect(),
        None,
    )?;

    Ok(())
}
