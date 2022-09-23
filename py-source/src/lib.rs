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
use zenoh_flow_python_common::configuration_into_py;
use zenoh_flow_python_common::from_pyerr_to_zferr;
use zenoh_flow_python_common::{DataSender, PythonState};

#[cfg(target_family = "unix")]
use libloading::os::unix::Library;
#[cfg(target_family = "windows")]
use libloading::Library;

#[cfg(target_family = "unix")]
static LOAD_FLAGS: std::os::raw::c_int =
    libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL;

pub static PY_LIB: &str = env!("PY_LIB");

#[derive(Debug)]
struct PySource(Library);

#[async_trait]
impl Source for PySource {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        // Preparing python
        pyo3::prepare_freethreaded_python();

        // Configuring wrapper + python source
        let my_state = Arc::new(Python::with_gil(|py| {
            match configuration {
                Some(configuration) => {
                    // Unwrapping configuration
                    let script_file_path = Path::new(
                        configuration["python-script"]
                            .as_str()
                            .ok_or(zferror!(ErrorKind::InvalidState))?,
                    );
                    let mut config = configuration.clone();

                    config["python-script"].take();
                    let py_config = config["configuration"].take();

                    // Convert configuration to Python
                    let py_config = configuration_into_py(py, py_config)
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                    // Load Python code
                    let code = read_file(script_file_path).unwrap(); //?;
                    let module = PyModule::from_code(
                        py,
                        &code,
                        &script_file_path.to_string_lossy(),
                        "source",
                    )
                    .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                    // Getting the correct python module
                    let source_class = module
                        .call_method0("register")
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                    let py_senders = PyDict::new(py);

                    for (id, output) in outputs.into_iter() {
                        let pyo3_tx = DataSender::from(output);
                        py_senders
                            .set_item(PyString::new(py, &id), &pyo3_tx.into_py(py))
                            .map_err(|e| from_pyerr_to_zferr(e, &py))?;
                    }

                    // Setting asyncio event loop
                    let asyncio = py.import("asyncio").unwrap();

                    let event_loop = asyncio.call_method0("new_event_loop").unwrap();
                    asyncio
                        .call_method1("set_event_loop", (event_loop,))
                        .unwrap();
                    let event_loop_hdl = Arc::new(PyObject::from(event_loop));

                    // Initialize Python Object
                    let py_source: PyObject = source_class
                        .call1((py_config, py_senders))
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?
                        .into();

                    Ok(PythonState {
                        module: Arc::new(source_class.into()),
                        py_state: Arc::new(py_source),
                        event_loop: event_loop_hdl,
                        asyncio_module: Arc::new(PyObject::from(asyncio)),
                    })
                }
                None => Err(zferror!(ErrorKind::InvalidState)),
            }
        })?);

        Ok(Some(Box::new(move || {
            let c_state = my_state.clone();
            async move {
                Python::with_gil(|py| {
                    let source_class = c_state.py_state.cast_as::<PyAny>(py)?;

                    let event_loop = c_state.event_loop.cast_as::<PyAny>(py)?;

                    let task_locals = TaskLocals::new(event_loop);

                    let py_future = source_class.call_method0("run")?;

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
zenoh_flow::export_source!(register);

fn load_self() -> Result<Library> {
    log::trace!("Python Source Wrapper loading Python {}", PY_LIB);

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
fn register() -> Result<Arc<dyn Source>> {
    let library = load_self()?;

    Ok(Arc::new(PySource(library)) as Arc<dyn Source>)
}

fn read_file(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}
