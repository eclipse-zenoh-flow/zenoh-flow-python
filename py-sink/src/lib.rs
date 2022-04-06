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

use async_trait::async_trait;
use pyo3::{prelude::*, types::PyModule};
use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::Configuration;
use zenoh_flow::{DataMessage, Node, Sink, State, ZFError, ZFResult};
use zenoh_flow_python_types::from_pyerr_to_zferr;
use zenoh_flow_python_types::utils::configuration_into_py;
use zenoh_flow_python_types::Context as PyContext;
use zenoh_flow_python_types::DataMessage as PyDataMessage;

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

#[derive(Debug)]
struct PySink(Library);

#[async_trait]
impl Sink for PySink {
    async fn run(
        &self,
        ctx: &mut zenoh_flow::Context,
        state: &mut State,
        input: DataMessage,
    ) -> ZFResult<()> {
        // Preparing python
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Preparing parameters
        let current_state = state.try_get::<PythonState>()?;
        let sink_class = current_state.module.as_ref().clone();
        let py_ctx = PyContext::from(ctx);
        let py_data = PyDataMessage::try_from(input)?;

        // Calling python
        sink_class
            .call_method1(
                py,
                "run",
                (
                    sink_class.clone(),
                    py_ctx,
                    current_state.py_state.as_ref().clone(),
                    py_data,
                ),
            )
            .map_err(|e| from_pyerr_to_zferr(e, &py))?;

        Ok(())
    }
}

impl Node for PySink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        // Preparing python
        pyo3::prepare_freethreaded_python();
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Configuring wrapper + python sink
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

                // Load Python code
                let code = read_file(script_file_path)?;
                let module =
                    PyModule::from_code(py, &code, &script_file_path.to_string_lossy(), "sink")
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?;
                // Getting the correct python module
                let sink_class: PyObject = module
                    .call_method0("register")
                    .map_err(|e| from_pyerr_to_zferr(e, &py))?
                    .into();

                // Initialize python state
                let state: PyObject = sink_class
                    .call_method1(py, "initialize", (sink_class.clone(), py_config))
                    .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                Ok(State::from(PythonState {
                    module: Arc::new(sink_class),
                    py_state: Arc::new(state),
                }))
            }
            None => Err(ZFError::InvalidState),
        }
    }

    fn finalize(&self, state: &mut State) -> ZFResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let current_state = state.try_get::<PythonState>()?;
        let sink_class = current_state.module.as_ref().clone();

        sink_class
            .call_method1(
                py,
                "finalize",
                (sink_class.clone(), current_state.py_state.as_ref().clone()),
            )
            .map_err(|e| from_pyerr_to_zferr(e, &py))?;

        Ok(())
    }
}

// Also generated by macro
zenoh_flow::export_sink!(register);

fn load_self() -> ZFResult<Library> {
    log::trace!("Python Sink Wrapper loading Python {}", PY_LIB);

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

fn register() -> ZFResult<Arc<dyn Sink>> {
    let library = load_self()?;
    Ok(Arc::new(PySink(library)) as Arc<dyn Sink>)
}

fn read_file(path: &Path) -> ZFResult<String> {
    Ok(fs::read_to_string(path)?)
}
