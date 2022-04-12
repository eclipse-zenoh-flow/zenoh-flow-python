//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use pyo3::{prelude::*, types::PyModule};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use zenoh_flow::runtime::message::DataMessage;
use zenoh_flow::Configuration;
use zenoh_flow::{
    async_std::sync::Arc, export_operator, types::ZFResult, InputToken, LocalDeadlineMiss, Node,
    NodeOutput, Operator, PortId, State,
};
use zenoh_flow::{Context, Data, ZFError};
use zenoh_flow_python_common::configuration_into_py;
use zenoh_flow_python_common::PythonState;
use zenoh_flow_python_common::{
    from_context_to_pyany, from_input_tokens_to_pydict, from_inputs_to_pydict,
    from_local_deadline_miss_to_pyany, from_outputs_to_pydict, from_pyany_to_or_result,
    from_pyany_to_run_result, from_pydict_to_input_tokens, from_pyerr_to_zferr,
};

#[cfg(target_family = "unix")]
use libloading::os::unix::Library;
#[cfg(target_family = "windows")]
use libloading::Library;

#[cfg(target_family = "unix")]
static LOAD_FLAGS: std::os::raw::c_int =
    libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL;

pub static PY_LIB: &str = env!("PY_LIB");

struct PyOperator(Library);

impl Operator for PyOperator {
    fn input_rule(
        &self,
        ctx: &mut Context,
        state: &mut State,
        tokens: &mut HashMap<PortId, InputToken>,
    ) -> ZFResult<bool> {
        // Getting tokens for conversion to Python
        let mut real_tokens = std::mem::take(tokens);

        // Preparing python environment
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Preparing parameters
        let current_state = state.try_get::<PythonState>()?;
        let op_class = current_state.module.as_ref().clone();

        let zf_types_module = current_state
            .py_zf_types
            .cast_as::<PyModule>(py)
            .map_err(|e| from_pyerr_to_zferr(e.into(), &py))?;

        let py_ctx = from_context_to_pyany(ctx, &py, zf_types_module)?;
        let py_tokens = from_input_tokens_to_pydict(&mut real_tokens, &py, zf_types_module)?;


        // Calling python code
        let ir_result: bool = op_class
            .call_method1(
                py,
                "input_rule",
                (
                    op_class.clone(),
                    py_ctx,
                    current_state.py_state.as_ref().clone(),
                    py_tokens,
                ),
            )
            .map_err(|e| from_pyerr_to_zferr(e, &py))?
            .extract(py)
            .map_err(|e| from_pyerr_to_zferr(e, &py))?;

        // // Getting back the tokens and update tokens
        *tokens = from_pydict_to_input_tokens(py_tokens, &py)?;

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

        let zf_types_module = current_state
            .py_zf_types
            .cast_as::<PyModule>(py)
            .map_err(|e| from_pyerr_to_zferr(e.into(), &py))?;

        let py_ctx = from_context_to_pyany(ctx, &py, zf_types_module)?;

        let py_data = from_inputs_to_pydict(inputs, &py, zf_types_module)?;


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
            .map_err(|e| from_pyerr_to_zferr(e, &py))?
            .into_ref(py);

            // Converting the results
        from_pyany_to_run_result(py_values, &py)


    }

    fn output_rule(
        &self,
        ctx: &mut Context,
        state: &mut State,
        mut outputs: HashMap<PortId, Data>,
        deadlinemiss: Option<LocalDeadlineMiss>,
    ) -> ZFResult<HashMap<PortId, NodeOutput>> {
        // Preparing python
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Preparing parameters
        let current_state = state.try_get::<PythonState>()?;
        let op_class = current_state.module.as_ref().clone();

        let zf_types_module = current_state
            .py_zf_types
            .cast_as::<PyModule>(py)
            .map_err(|e| from_pyerr_to_zferr(e.into(), &py))?;

        let py_ctx = from_context_to_pyany(ctx, &py, zf_types_module)?;
        let py_data = from_outputs_to_pydict(&mut outputs, &py)?;

        let py_values = match deadlinemiss {
            Some(deadlinemiss) => op_class
                .call_method1(
                    py,
                    "output_rule",
                    (
                        op_class.clone(),
                        py_ctx,
                        current_state.py_state.as_ref().clone(),
                        py_data,
                        from_local_deadline_miss_to_pyany(&deadlinemiss, &py, zf_types_module)?,
                    ),
                )
                .map_err(|e| from_pyerr_to_zferr(e, &py))?
                .into_ref(py),
            None => op_class
                .call_method1(
                    py,
                    "output_rule",
                    (
                        op_class.clone(),
                        py_ctx,
                        current_state.py_state.as_ref().clone(),
                        py_data,
                    ),
                )
                .map_err(|e| from_pyerr_to_zferr(e, &py))?
                .into_ref(py),
        };

        // Converting the results
        from_pyany_to_or_result(py_values, &py)
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
                let py_config = configuration_into_py(py, py_config)
                    .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                let py_zf_types = PyModule::import(py, "zenoh_flow.types")
                    .map_err(|e| from_pyerr_to_zferr(e, &py))?
                    .to_object(py);

                // Load Python code
                let code = read_file(script_file_path)?;
                let module =
                    PyModule::from_code(py, &code, &script_file_path.to_string_lossy(), "op")
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                // Getting the correct python module
                let op_class: PyObject = module
                    .call_method0("register")
                    .map_err(|e| from_pyerr_to_zferr(e, &py))?
                    .into();

                // Initialize python state
                let state: PyObject = op_class
                    .call_method1(py, "initialize", (op_class.clone(), py_config))
                    .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                Ok(State::from(PythonState {
                    module: Arc::new(op_class),
                    py_state: Arc::new(state),
                    py_zf_types: Arc::new(py_zf_types),
                }))
            }
            None => Err(ZFError::InvalidState),
        }
    }

    fn finalize(&self, state: &mut State) -> ZFResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let current_state = state.try_get::<PythonState>()?;
        let op_class = current_state.module.as_ref().clone();

        op_class
            .call_method1(
                py,
                "finalize",
                (op_class.clone(), current_state.py_state.as_ref().clone()),
            )
            .map_err(|e| from_pyerr_to_zferr(e, &py))?;

        Ok(())
    }
}

export_operator!(register);

fn load_self() -> ZFResult<Library> {
    log::trace!("Python Operator Wrapper loading Python {}", PY_LIB);
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

fn read_file(path: &Path) -> ZFResult<String> {
    Ok(fs::read_to_string(path)?)
}
