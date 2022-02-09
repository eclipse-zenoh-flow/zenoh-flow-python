..
.. Copyright (c) 2017, 2022 ADLINK Technology Inc.
..
.. This program and the accompanying materials are made available under the
.. terms of the Eclipse Public License 2.0 which is available at
.. http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
.. which is available at https://www.apache.org/licenses/LICENSE-2.0.
..
.. SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
..
.. Contributors:
..   ADLINK zenoh team, <zenoh@adlink-labs.tech>
..

=================
zenoh-flow-python
=================


The *zenoh-flow-python* library provides a set of Python interfaces to write your
operators, source and sink for Zenoh Flow.


Note that API it's NOT meant to be used directly, instead your operators, sink
and sources have to implement the methods provided by the different classes.
Then such nodes need to be loaded by a Zenoh Flow Runtime

.. toctree::
    :maxdepth: 1

    Zenoh Flow Interfaces <zenoh-flow-nodes>
    Zenoh Flow API <zenoh-flow-api>
