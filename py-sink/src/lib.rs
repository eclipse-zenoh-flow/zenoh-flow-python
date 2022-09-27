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
use pyo3::types::{PyDict, PyString};
use pyo3::{prelude::*, types::PyModule};
use pyo3_asyncio::TaskLocals;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use zenoh_flow::prelude::*;
use zenoh_flow_python_common::{configuration_into_py, DataReceiver};
use zenoh_flow_python_common::{context_into_py, from_pyerr_to_zferr};
use zenoh_flow_python_common::{get_python_input_callbacks, PythonState};

#[cfg(target_family = "unix")]
use libloading::os::unix::Library;
#[cfg(target_family = "windows")]
use libloading::Library;

#[cfg(target_family = "unix")]
static LOAD_FLAGS: std::os::raw::c_int =
    libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL;

pub static PY_LIB: &str = env!("PY_LIB");

#[derive(Debug)]
struct PySink(Library);

#[async_trait]
impl Sink for PySink {
    async fn setup(
        &self,
        ctx: &mut Context,
        configuration: &Option<Configuration>,
        inputs: Inputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        // prepare python
        pyo3::prepare_freethreaded_python();
        let my_state = Arc::new(Python::with_gil(|py| {
            match configuration {
                Some(configuration) => {
                    // Unwrapping configuration
                    let script_file_path = Path::new(
                        configuration["python-script"]
                            .as_str()
                            .ok_or_else(|| zferror!(ErrorKind::InvalidState))?,
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
                    let sink_class = module
                        .call_method0("register")
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                    let py_receivers = PyDict::new(py);

                    for (id, input) in inputs.iter() {
                        let pyo3_rx = DataReceiver::from(input);
                        py_receivers
                            .set_item(PyString::new(py, &id), &pyo3_rx.into_py(py))
                            .map_err(|e| from_pyerr_to_zferr(e, &py))?;
                    }

                    // Setting asyncio event loop
                    let asyncio = py.import("asyncio").unwrap();

                    let event_loop = asyncio.call_method0("new_event_loop").unwrap();
                    asyncio
                        .call_method1("set_event_loop", (event_loop,))
                        .unwrap();
                    let event_loop_hdl = Arc::new(PyObject::from(event_loop));
                    let py_ctx =
                        context_into_py(&py, ctx).map_err(|e| from_pyerr_to_zferr(e, &py))?;

                    // Initialize Python Object
                    let py_sink: PyObject = sink_class
                        .call1((py_ctx, py_config, py_receivers))
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?
                        .into();

                    let py_state = PythonState {
                        module: Arc::new(sink_class.into()),
                        py_state: Arc::new(py_sink),
                        event_loop: event_loop_hdl,
                        asyncio_module: Arc::new(PyObject::from(asyncio)),
                    };

                    // Callback setup
                    let callback_hashmap = get_python_input_callbacks(&py, py_ctx, inputs)?;

                    for (input, callback) in callback_hashmap.into_iter() {
                        ctx.register_input_callback(input, callback)
                    }

                    Ok(py_state)
                }
                None => Err(zferror!(ErrorKind::InvalidState)),
            }
        })?);

        Ok(Some(Box::new(move || {
            let c_state = my_state.clone();
            async move {
                Python::with_gil(|py| {
                    let sink_class = c_state.py_state.cast_as::<PyAny>(py)?;

                    let event_loop = c_state.event_loop.cast_as::<PyAny>(py)?;

                    let task_locals = TaskLocals::new(event_loop);

                    let py_future = sink_class.call_method0("iteration")?;

                    let fut = pyo3_asyncio::into_future_with_locals(&task_locals, py_future)?;
                    pyo3_asyncio::async_std::run_until_complete(event_loop, fut)
                })
                .map_err(|e| Python::with_gil(|py| from_pyerr_to_zferr(e, &py)))?;
                Ok(())
            }
        })))
    }
}

// Also generated by macro
zenoh_flow::export_sink!(register);

fn load_self() -> Result<Library> {
    log::trace!("Python Sink Wrapper loading Python {}", PY_LIB);

    // Very dirty hack! We explicit load the python library!
    let lib_name = libloading::library_filename(PY_LIB);
    unsafe {
        #[cfg(target_family = "unix")]
        let lib = Library::open(Some(lib_name), LOAD_FLAGS)?;

        #[cfg(target_family = "windows")]
        let lib = Library::new(lib_name)?;

        Ok(lib)
    }
}

fn register() -> Result<Arc<dyn Sink>> {
    let library = load_self()?;
    Ok(Arc::new(PySink(library)) as Arc<dyn Sink>)
}

fn read_file(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}
