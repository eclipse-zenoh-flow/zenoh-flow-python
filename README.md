[![Discussion](https://img.shields.io/badge/discussion-on%20github-blue)](https://github.com/eclipse-zenoh/roadmap/discussions)
[![Discord](https://img.shields.io/badge/chat-on%20discord-blue)](https://discord.gg/vSDSpqnbkm)
[![Eclipse CI](https://ci.eclipse.org/zenoh/buildStatus/icon?job=zenoh-flow-python-nightly&subject=Eclipse%20CI)](https://ci.eclipse.org/zenoh/view/Zenoh%20Flow/job/zenoh-flow-python-nightly/)
[![CI](https://github.com/eclipse-zenoh/zenoh-flow-python/actions/workflows/ci.yml/badge.svg)](https://github.com/eclipse-zenoh/zenoh-flow-python/actions/workflows/ci.yml)

<img src="https://raw.githubusercontent.com/eclipse-zenoh/zenoh/master/zenoh-dragon.png" height="150">


# Zenoh-Flow Python bindings
[Zenoh-Flow](https://github.com/eclipse-zenoh-flow/zenoh-flow) provides a Zenoh-based data flow programming framework for computations that span from the cloud to the device.

:warning: **This software is still in alpha status and we do not recommend using it in production.**

-----------

## Requirements

- Rust: see the [installation page](https://www.rust-lang.org/tools/install)
- Python >= 3.8
- pip >= 22
- virtualenv


## Installation

### Install the Python package: `zenoh_flow_python`

If it's not already the case, start by activating a virtual environment:

```bash
$ python3 -m virtualenv venv
$ source venv/bin/activate
```

⚠️ On **macOS** the Zenoh-Flow wrappers have to patch the `sys.path` variable of the Python interpreter with the location of the site-packages folder of the currently active virtual environment. To do so, we rely on the `$VIRTUAL_ENV` environment variable. If your favorite environment manager does not set this variable then the `zenoh_flow_python` module will not be found when launching your flow.

Build the Python Wheel **within** a Python virtual environment.

```bash
(venv) $ git clone https://github.com/eclipse-zenoh-flow/zenoh-flow-python
(venv) $ cd zenoh-flow-python/zenoh-flow-python
(venv) $ pip3 install -r requirements-dev.txt
(venv) $ maturin build --release
(venv) $ pip install ./target/wheels/<there should only be one .whl file here>
```

#### Build the wrapper shared libraries & generate a configuration for the Zenoh-Flow runtime

Build the Python wrappers.

```bash
$ cargo build --release -p zenoh-flow-python-operator-wrapper -p zenoh-flow-python-sink-wrapper -p zenoh-flow-python-source-wrapper
```


## FAQ and Troubleshooting

### Cannot load library, no extension found for files of type < py >

This error indicates that the Zenoh-Flow runtime was not properly configured to support nodes written in Python.

You need to change the configuration of Zenoh-Flow to let it know how to load Python scripts.

*If* you launched the Zenoh-Flow runtime in a **standalone** fashion, you need to provide a configuration that contains the following:

```yaml
name: my-zenoh-flow

extensions:
  - file_extension: py
    libraries:
      # Linux
      sink: /path/to/zenoh-flow-python/target/release/libzenoh_flow_python_sink_wrapper.so
      operator: /path/to/zenoh-flow-python/target/release/libzenoh_flow_python_operator_wrapper.so
      source: /path/to/zenoh-flow-python/target/release/libzenoh_flow_python_source_wrapper.so
      # macOS
      # sink: /path/to/zenoh-flow-python/target/release/libzenoh_flow_python_sink_wrapper.dylib
      # operator: /path/to/zenoh-flow-python/target/release/libzenoh_flow_python_operator_wrapper.dylib
      # source: /path/to/zenoh-flow-python/target/release/libzenoh_flow_python_source_wrapper.dylib
```

*If* you launched Zenoh-Flow as a **Zenoh plugin**, you need to update the Zenoh configuration with the following:

```json
{
    "plugins": {
        "zenoh_flow": {
            "name": "my-zenoh-flow",
            "extensions": [
                {
                    "file_extension": "py",
                    "libraries": {
                        // Linux
                        "operator": "/path/to/zenoh-flow-python/target/release/libzenoh_flow_python_operator_wrapper.so",
                        "sink": "/path/to/zenoh-flow-python/target/release/libzenoh_flow_python_sink_wrapper.so",
                        "source": "/path/to/zenoh-flow-python/target/release/libzenoh_flow_python_source_wrapper.so",
                        // macOS
                        // "operator": "/path/to/zenoh-flow-python/target/release/libzenoh_flow_python_operator_wrapper.dylib",
                        // "sink": "/path/to/zenoh-flow-python/target/release/libzenoh_flow_python_sink_wrapper.dylib",
                        // "source": "/path/to/zenoh-flow-python/target/release/libzenoh_flow_python_source_wrapper.dylib",
                    }
                }
            ]
        }
    }
}
```

### Failed to load `zenoh_flow_python` module

First, check that you have activated your virtual environment before launching the Zenoh-Flow runtime (be it through the standalone daemon, runtime or dedicated Zenoh plugin). **Make sure that the environment variable `$VIRTUAL_ENV` is set**. 

⚠️ *If your environment manager does not set this variable, please open an issue specifying your setup*.

If your virtual environment is activated, check that the Zenoh-Flow Python package is indeed installed:
1. Open a terminal.
2. Enter the Python interpreter:
   ```bash
   python
   ```
3. Try to import Zenoh-Flow Python:
   ```bash
   import zenoh_flow_python
   ```
   
If you still have the above error when launching your data flow, try reinstalling the package:
1. Open a terminal.
2. `cd` into `zenoh-flow-python/zenoh-flow-python`.
3. Build:
   ```bash
   maturin build --release
   ```
4. Install:
   ```bash
   pip install ../target/wheels/*.whl --force-reinstall
   ```
   
Try again relaunching your data flow.
