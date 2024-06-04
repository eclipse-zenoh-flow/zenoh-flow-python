//
// Copyright © 2022 ZettaScale Technology
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

use std::sync::Arc;

use async_trait::async_trait;
use pyo3::{types::PyModule, PyAny, Python};
use zenoh_flow_nodes::{
    prelude as zf,
    prelude::{anyhow, export_sink, Sink},
};
use zenoh_flow_python::{configuration_into_py, Context, Inputs, PythonState};

#[export_sink]
struct ZenohFlowPythonSink {
    state: Arc<PythonState>,
}

#[async_trait]
impl Sink for ZenohFlowPythonSink {
    async fn new(
        context: zf::Context,
        configuration: zf::Configuration,
        inputs: zf::Inputs,
    ) -> zf::Result<Self> {
        pyo3_pylogger::register(&format!("zenoh_flow_python_sink::{}", context.node_id()));
        let _ = tracing_subscriber::fmt::try_init();

        let state = Arc::new(Python::with_gil(|py| {
            // NOTE: See https://github.com/PyO3/pyo3/issues/1741#issuecomment-1191125053
            //
            // On macOS, the site-packages folder of the current virtual environment is not added to the `sys.path`
            // making it impossible to load the modules that were installed on it.
            #[cfg(target_os = "macos")]
            if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
                let version_info = py.version_info();
                let sys = py.import("sys").unwrap();
                let sys_path = sys.getattr("path").unwrap();
                let site_packages_dir = format!(
                    "{}/lib/python{}.{}/site-packages",
                    venv, version_info.major, version_info.minor
                );

                tracing::debug!("Adding virtual environment site-packages folder to Python interpreter path: {site_packages_dir}");

                sys_path
                    .call_method1("append", (site_packages_dir,))
                    .unwrap();
            }

            let py_configuration = configuration_into_py(py, configuration)
                .map_err(|e| anyhow!("Failed to convert `Configuration` to `PyObject`: {e:?}"))?;

            // Setting asyncio event loop
            let asyncio = py.import("asyncio").unwrap();
            let event_loop = asyncio.call_method0("new_event_loop").unwrap();
            asyncio
                .call_method1("set_event_loop", (event_loop,))
                .unwrap();
            let task_locals = pyo3_asyncio::TaskLocals::new(event_loop);

            let user_code = std::fs::read_to_string(context.library_path()).map_err(|e| {
                anyhow!(
                    "Failed to read < {} >: {e:?}",
                    context.library_path().display()
                )
            })?;

            let python_module = PyModule::from_code(
                py,
                &user_code,
                &context.library_path().to_string_lossy(),
                "zenoh_flow_python_sink",
            )
            .map_err(|e| {
                anyhow!(
                    "Failed to create `PyModule` from script < {} >: {e:?}",
                    context.library_path().display()
                )
            })?;

            let sink_class = python_module
                .call_method0("register")
                .map_err(|e| anyhow!("Call to `register` failed with: {e:?}"))?;

            // NOTE: `call1` will call the object pointed at by `sink_class` with the provided parameters.  This
            // translates to creating a new instance of the class.
            let sink_instance = sink_class
                .call1((
                    Context::from(context),
                    py_configuration,
                    Inputs::from(inputs),
                ))
                .map_err(|e| anyhow!("Failed to create a Sink instance: {e:?}"))?;

            zf::Result::Ok(PythonState {
                node_instance: Arc::new(sink_instance.into()),
                task_locals: Arc::new(task_locals),
            })
        })?);

        Ok(Self { state })
    }
}

#[async_trait]
impl zf::Node for ZenohFlowPythonSink {
    async fn iteration(&self) -> zf::Result<()> {
        tracing::debug!("iteration");

        Python::with_gil(|py| {
            let sink_instance = self
                .state
                .node_instance
                .downcast::<PyAny>(py)
                .map_err(|e| anyhow!("Failed to downcast Sink instance to `PyAny`: {e:?}"))?;

            let iteration_coroutine = sink_instance
                .call_method0("iteration")
                .map_err(|e| anyhow!("Call to `iteration` failed with: {e:?}"))?;

            let iteration =
                pyo3_asyncio::into_future_with_locals(&self.state.task_locals, iteration_coroutine)
                    .map_err(|e| {
                        anyhow!(
                        "(pyo3-asyncio) Failed to transform Python coroutine to Rust future: {e:?}"
                    )
                    })?;

            let _ = pyo3_asyncio::async_std::run_until_complete(
                self.state.task_locals.event_loop(py),
                iteration,
            )
            .map_err(|e| anyhow!("Iteration failed with: {e:?}"))?;

            Ok(())
        })
    }
}
