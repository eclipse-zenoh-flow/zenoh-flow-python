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

fn get_py_lib_name() -> String {
    let config = pyo3_build_config::get();
    match &config.lib_name {
        Some(name) => name.clone(),
        None => panic!("Unable to find Python version"),
    }
}

fn main() {
    let py = get_py_lib_name();
    println!("cargo:rustc-env=PY_LIB={}", py);
    println!("cargo:rustc-link-lib=dylib={}", py);
}
