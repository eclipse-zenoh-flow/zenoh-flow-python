..
.. Copyright (c) 2022 ZettaScale Technology
..
.. This program and the accompanying materials are made available under the
.. terms of the Eclipse Public License 2.0 which is available at
.. http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
.. which is available at https://www.apache.org/licenses/LICENSE-2.0.
..
.. SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
..
.. Contributors:
..   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
..


Zenoh Flow interfaces references
================================

Note that, this API is NOT meant to be used directly.
Instead your operators, sink and sources MUST implement the methods provided by the following classes.
Only then, such nodes can to be loaded by a Zenoh Flow Runtime

Source
------
.. autoclass:: zenoh_flow.interfaces.Source
    :members:


Sink
----
.. autoclass:: zenoh_flow.interfaces.Sink
    :members:


Operator
--------
.. autoclass:: zenoh_flow.interfaces.Operator
    :members:
