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

use pyo3::prelude::*;
use zenoh_flow_python_common::{DataMessage, DataReceiver, DataSender};

#[pymodule]
fn zenoh_flow(_py: Python, m: &PyModule) -> PyResult<()> {
    // m.add_class::<PyReceiver>()?;
    // m.add_class::<PySender>()?;
    m.add_class::<DataSender>()?;
    m.add_class::<DataReceiver>()?;
    m.add_class::<DataMessage>()?;

    Ok(())
}
