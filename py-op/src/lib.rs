//
// Copyright (c) 2017, 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//

use pyo3::{prelude::*, types::PyModule};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs;
use std::path::Path;
use zenoh_flow::runtime::message::DataMessage;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::Configuration;
use zenoh_flow::{
    async_std::sync::Arc, export_operator, types::ZFResult, LocalDeadlineMiss, Node, NodeOutput,
    Operator, PortId, State, Token,
};
use zenoh_flow::{Context, Data, ZFError};
use zenoh_flow_python_types::utils::{configuration_into_py, outputs_from_py, tokens_into_py};
use zenoh_flow_python_types::Context as PyContext;
use zenoh_flow_python_types::Inputs as PyInputs;
use zenoh_flow_python_types::LocalDeadlineMiss as PyLocalDeadlineMiss;
use zenoh_flow_python_types::Outputs as PyOutputs;
use zenoh_flow_python_types::Token as PyToken;

#[cfg(target_family = "unix")]
use libloading::os::unix::Library;
#[cfg(target_family = "windows")]
use libloading::Library;

#[cfg(target_family = "unix")]
static LOAD_FLAGS: std::os::raw::c_int =
    libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL;

pub static PY_LIB: &str = env!("PY_LIB");

#[derive(ZFState, Clone)]
struct PythonState {
    pub module: Arc<PyObject>,
    pub py_state: Arc<PyObject>,
}
unsafe impl Send for PythonState {}
unsafe impl Sync for PythonState {}

impl std::fmt::Debug for PythonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PythonState").finish()
    }
}

struct PyOperator(Library);

impl Operator for PyOperator {
    fn input_rule(
        &self,
        ctx: &mut Context,
        state: &mut State,
        tokens: &mut HashMap<PortId, Token>,
    ) -> ZFResult<bool> {
        // Getting tokens for conversion to Python
        let real_tokens = std::mem::replace(tokens, HashMap::new());

        // Preparing python environment
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Preparing parameters
        let current_state = state.try_get::<PythonState>()?;
        let op_class = current_state.module.as_ref().clone();
        let py_ctx = PyContext::from(ctx);
        let py_tokens = tokens_into_py(py, real_tokens);

        // Calling python code
        let ir_result: bool = op_class
            .call_method1(
                py,
                "input_rule",
                (
                    op_class.clone(),
                    py_ctx,
                    current_state.py_state.as_ref().clone(),
                    &py_tokens,
                ),
            )
            .map_err(|e| ZFError::InvalidData(e.to_string()))?
            .extract(py)
            .map_err(|e| ZFError::InvalidData(e.to_string()))?;

        // Getting back the tokens
        let py_tokens: HashMap<String, PyToken> = py_tokens
            .extract(py)
            .map_err(|e| ZFError::InvalidData(e.to_string()))?;

        // Converting the tokens to the rust type
        let new_tokens = {
            let mut n_tokens = HashMap::new();
            for (id, t) in py_tokens {
                n_tokens.insert(id.into(), t.into());
            }

            n_tokens
        };

        // Update tokens
        *tokens = new_tokens;

        Ok(ir_result)
    }

    fn run(
        &self,
        ctx: &mut Context,
        state: &mut State,
        inputs: &mut HashMap<PortId, DataMessage>,
    ) -> ZFResult<HashMap<PortId, Data>> {
        // Prepare Python
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Preparing parameters
        let current_state = state.try_get::<PythonState>()?;
        let op_class = current_state.module.as_ref().clone();
        let py_ctx = PyContext::from(ctx);
        let py_data = PyInputs::try_from(inputs)?;

        // Call python copde
        let py_values = op_class
            .call_method1(
                py,
                "run",
                (
                    op_class.clone(),
                    py_ctx,
                    current_state.py_state.as_ref().clone(),
                    py_data,
                ),
            )
            .map_err(|e| ZFError::InvalidData(e.to_string()))?;

        // Converting the results
        let values = outputs_from_py(py, py_values.into())?;

        Ok(values.try_into()?)
    }

    fn output_rule(
        &self,
        ctx: &mut Context,
        state: &mut State,
        outputs: HashMap<PortId, Data>,
        deadlinemiss: Option<LocalDeadlineMiss>,
    ) -> ZFResult<HashMap<PortId, NodeOutput>> {
        // Preparing python
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Preparing parameters
        let current_state = state.try_get::<PythonState>()?;
        let op_class = current_state.module.as_ref().clone();
        let py_ctx = PyContext::from(ctx);
        let py_data = PyOutputs::try_from(outputs)?;
        let deadline_miss = PyLocalDeadlineMiss::from(deadlinemiss);

        // Calling pthon code
        let py_values = op_class
            .call_method1(
                py,
                "output_rule",
                (
                    op_class.clone(),
                    py_ctx,
                    current_state.py_state.as_ref().clone(),
                    py_data,
                    deadline_miss,
                ),
            )
            .map_err(|e| ZFError::InvalidData(e.to_string()))?;

        // Converting the results
        let py_values = outputs_from_py(py, py_values.into())?;

        // Generating the rust output
        let rust_values: HashMap<PortId, Data> = py_values.try_into()?;

        let mut results = HashMap::with_capacity(rust_values.len());
        for (k, v) in rust_values {
            results.insert(k, NodeOutput::Data(v));
        }

        Ok(results)
    }
}

impl Node for PyOperator {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        // Preparing python
        pyo3::prepare_freethreaded_python();
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Configuring wrapper + python operator
        match configuration {
            Some(configuration) => {
                // Unwrapping configuration
                let script_file_path = Path::new(
                    configuration["python-script"]
                        .as_str()
                        .ok_or(ZFError::InvalidState)?,
                );
                let mut config = configuration.clone();
                config["python-script"].take();
                let py_config = config["configuration"].take();

                // Convert configuration to Python
                let py_config = configuration_into_py(py, py_config);

                // Load Python code
                let code = read_file(script_file_path);
                let module = PyModule::from_code(py, &code, "op.py", "op")
                    .map_err(|e| ZFError::InvalidData(e.to_string()))?;

                // Getting the correct python module
                let op_class: PyObject = module
                    .call_method0("register")
                    .map_err(|e| ZFError::InvalidData(e.to_string()))?
                    .into();

                // Initialize python state
                let state: PyObject = op_class
                    .call_method1(py, "initialize", (op_class.clone(), py_config))
                    .map_err(|e| ZFError::InvalidData(e.to_string()))?
                    .into();

                Ok(State::from(PythonState {
                    module: Arc::new(op_class),
                    py_state: Arc::new(state),
                }))
            }
            None => Err(ZFError::InvalidState),
        }
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

export_operator!(register);

fn load_self() -> ZFResult<Library> {
    // Very dirty hack!
    let lib_name = libloading::library_filename(PY_LIB);
    unsafe {
        #[cfg(target_family = "unix")]
        let lib = Library::open(Some(lib_name), LOAD_FLAGS)?;

        #[cfg(target_family = "windows")]
        let lib = Library::new(lib_name)?;

        Ok(lib)
    }
}

fn register() -> ZFResult<Arc<dyn Operator>> {
    let library = load_self()?;

    Ok(Arc::new(PyOperator(library)) as Arc<dyn Operator>)
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}
