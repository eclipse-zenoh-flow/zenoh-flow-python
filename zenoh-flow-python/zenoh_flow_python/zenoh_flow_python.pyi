#
# Copyright © 2022 ZettaScale Technology
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
#

from typing import final


@final
class InstanceId:
    """The unique identifier associated with a data flow instance."""
    def __str__(self) -> str: ...


@final
class RuntimeId:
    """The unique identifier associated with a Zenoh-Flow Runtime."""
    def __str__(self) -> str: ...


@final
class Context:
    """The execution context of a data flow instance."""
    @property
    def data_flow_name(self) -> str:
        """
        Returns the name of the data flow.

        All instances of the same data flow will share the same name. Their
        data flow instance id will however be unique to each.
        """

    @property
    def data_flow_instance_id(self) -> InstanceId:
        """
        Returns the unique identifier of this data flow instance.
        """

    @property
    def runtime_id(self) -> RuntimeId:
        """
        Returns the unique identifier of the Zenoh-Flow runtime managing the
        data flow instance.
        """

    @property
    def library_path(self) -> pathlib.Path:
        """
        Returns the path of the Python library currently running.

        This path will point to the Python code, not to the shared library
        wrapping it.
        """

    @property
    def node_id(self) -> str:
        """Returns the node unique identifier in the data flow."""


@final
class Timestamp:
    """
    A timestamp made of a NTP64 and a Hybrid Logical Clock (HLC) unique
    identifier.
    """

    @property
    def time(self) -> int:
        """Returns the number of milliseconds elapsed since UNIX EPOCH."""

    @property
    def id(self) -> str:
        """
        Returns the unique identifier of the HLC that generated the timestamp.

        The value is a hexadecimal representation.
        """


@final
class LinkMessage:
    """
    A message received from an InputRaw.

    A message is composed of two parts: a `payload` and a `Timestamp`. The
    payload is received as bytes and needs to be deserialised.
    """

    @property
    def payload(self) -> bytes:
        """
        Returns the payload, as bytes, associated with this message.
        """

    @property
    def timestamp(self) -> Timestamp:
        """
        Returns the Timestamp associated with this message.
        """


@final
class InputRaw:
    """
    A raw Zenoh-Flow Input, receiving serialised payload from upstream nodes.
    """
    async def recv_async(self) -> LinkMessage:
        """
        Retrieves, asynchronously, a message from the raw Input.

        # Exception

        An exception will be raised if the underlying channel is disconnected. A
        channel is disconnected only if the process holding the sending end is
        stopped (voluntarily or not).
        """

    def try_recv(self) -> LinkMessage | None:
        """
        Attempts to retrieve a message from the raw Input.

        If the channel is empty, this method will return None.

        # Exception

        An exception will be raised if the underlying channel is disconnected. A
        channel is disconnected only if the process holding the sending end is
        stopped (voluntarily or not).
        """

    def port_id(self) -> str:
        """
        Returns the port id associated with this raw Input.
        """


@final
class Inputs:
    """
    The channels *receiving* data from upstream nodes.
    """
    def take_raw(self, port_id: str) -> InputRaw:
        """
        Returns the raw Input associated with the provided port id.

        Note that the port id must be an exact match to what is written in the
        data flow descriptor.

        # Exception

        If no Input is associated with this port id, an exception is raised.
        """


@final
class OutputRaw:
    """
    A raw Zenoh-Flow Output, sending serialised payload to downstream nodes.
    """
    async def send_async(self, payload: bytes, timestamp_ms: int | None) -> None:
        """
        Sends, asynchronously, a payload on the raw Output.

        If a timestamp is provided, it will be interpreted as the number of
        milliseconds that elapsed since UNIX_EPOCH.

        If no timestamp is provided, a new timestamp will be generated by the
        Hybrid Logical Clock (HLC) used by the Zenoh-Flow runtime on which the
        node is executed.

        # Exception

        An exception will be raised if any of the underlying channels is
        disconnected. A channel is disconnected only if the process holding the
        receiving end is stopped (voluntarily or not).
        """

    def try_send(self, payload: bytes, timestamp_ms: Timestamp | None) -> None:
        """
        Attempts to send a payload on the raw Output.

        If a timestamp is provided, it will be interpreted as the number of
        milliseconds that elapsed since UNIX_EPOCH.

        If no timestamp is provided, a new timestamp will be generated by the
        Hybrid Logical Clock (HLC) used by the Zenoh-Flow runtime on which the
        node is executed.

        # Exceptions

        An exception will be raised if:
        1. Any of the underlying channels is full, hence preventing sending the
           payload.
        2. Any of the underlying channels is disconnected. A channel is
           disconnected only if the process holding the receiving end is stopped
           (voluntarily or not).
        """

    def port_id(self) -> str:
        """
        Returns the port id associated with this raw Output.
        """


@final
class Outputs:
    """
    The channels *sending* data to downstream nodes.
    """
    def take_raw(self, port_id: str) -> OutputRaw:
        """
        Returns the raw Output associated with the provided port id.

        Note that the port id must be an exact match to what is written in the
        data flow descriptor.

        # Exception

        If no Output is associated with this port id, an exception is raised.
        """
