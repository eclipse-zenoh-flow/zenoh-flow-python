# Python Zenoh Flow API

[![Join the chat at https://gitter.im/atolab/zenoh-flow](https://badges.gitter.im/atolab/zenoh-flow.svg)](https://gitter.im/atolab/zenoh-flow?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

[Zenoh Flow](https://github.com/eclipse-zenoh/zenoh-flow) provides a Zenoh-based dataflow programming framework for computations that span from the cloud to the device.

:warning: **This software is still in alpha status and should _not_ be used in production. Breaking changes are likely to happen and the API is not stable.**

-----------

### Requirements

- Python >= 3.7
- pip >= 22
- virtualenv




### How to build

Create and activate a python virtual environment:

```bash
$ python3 -m virtualenv venv
$ source venv/bin/activate
```

Build the Python Wheel

```bash
(venv) $ cd zenoh-flow-python
(venv) $ pip3 install -r requirements-dev.txt
(venv) $ maturin build --release
```

On a separate terminal install the wheel.

```bash
$ pip install ./target/wheels/<there should only be one .whl file here>
```

### Run an example

Clone and build the [Zenoh Flow runtime](https://github.com/eclipse-zenoh/zenoh-flow), and then use the provided `py-pipeline.yml` example to run it.

You may need to update the paths in `py-pipeline.yml` file and in `loader-config.yml`.

```bash
$ /path/to/zenoh-flow-examples/target/release/runtime -r foo -g ../py-pipeline.yml -l ../loader-config.yml
```