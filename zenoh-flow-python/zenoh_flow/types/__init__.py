#
# Copyright (c) 2022 ZettaScale Technology
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

from zenoh_flow import RawInput, RawOutput
from typing import Callable, Any, TypeVar, Optional, Dict


T = TypeVar("T")


class Context(object):
    """
    A Zenoh Flow context.
    Zenoh Flow context provides access to runtime and flow information to
    the operator.

    The context allows for registering callbacks in inputs and outputs.
    """

    def __init__(
        self, runtime_uuid: str, flow_name: str, instance_uuid: str
    ):
        self.runtime_uuid = runtime_uuid
        """ UUID of the runtime where the node is running."""
        self.flow_name = flow_name
        """Flow of which the node is part."""
        self.instance_uuid = instance_uuid
        """UUID of the flow instance the node is associated."""

    def __repr__(self):
        return self.__str__()

    def __str__(self):
        return (
            f"Context(runtime_uuid={self.runtime_uuid}, "
            + f"flow_name={self.flow_name}, "
            + f"instance_uuid={self.instance_uuid})"
        )


class Timestamp(object):
    """
    The Zenoh (Flow) timestamp.

    Attributes:
        ntp     NTP Timestamp
        id      UUID associated with the Timestamp producer.
    """

    def __init__(self, ntp: int, id: str):
        self.ntp = ntp
        self.id = id

    def __repr__(self):
        return self.__str__()

    def __str__(self):
        return f"Timestamp(ntp={self.ntp}, id={self.id})"


class Message:
    """
    A Zenoh-Flow message: a timestamp and optional data.

    If the message is a `Watermark` then no data is associated and
    `get_data` will return an empty list.
    """

    def __init__(self, data: Any, ts: int):
        self.data = data
        self.timestamp = ts


class Input:
    """
    Channel that receives data from upstream nodes.
    """

    def __init__(
        self, inner: RawInput, input_type: T, deserializer: Callable[[bytes], T]
    ):
        self.__deserializer = deserializer
        self.__inner = inner
        self.__type = input_type

    async def recv(self) -> Message:
        """
        Returns the first `DataMessage` that was received, *asynchronously*,
        on any of the channels associated with this Input.

        If several `DataMessage` are received at the same time,
        one is randomly selected.
        """
        data_msg = await self.__inner.recv()
        data = self.__deserializer(data_msg.data)
        msg = Message(data, data_msg.ts)
        return msg

    def port_id(self) -> str:
        """
        Returns the ID associated with this `Input`.
        """
        return self.__inner.port_id()


class Output:
    """
    Channels that sends data to downstream nodes.
    """

    def __init__(
        self, inner: RawOutput, output_type: T, serializer: Callable[[T], bytes]
    ):
        self.__serializer = serializer
        self.__inner = inner
        self.__type = output_type

    async def send(self, data: T, ts: Optional[int] = None):
        """
        Send, *asynchronously*, the data on all channels.

        If no timestamp is provided, the current timestamp
        — as per the HLC — is taken.

        If an error occurs while sending the message on a channel,
        we still try to send it on the remaining channels.
        For each failing channel, an error is logged and counted for.
        """
        ser_data = self.__serializer(data)
        return await self.__inner.send(ser_data, ts)

    def port_id(self) -> str:
        """
        Returns the ID associated with this `Output`.
        """
        return self.__inner.port_id()


class Inputs:
    """
    The `Inputs` structure contains all the receiving channels
    we created for a `Sink` or an `Operator`.
    """

    def __init__(self, inputs: Dict[str, RawInput]):
        self.__inputs = inputs

    def take(
        self, port_id: str, input_type: T, deserializer: Callable[[bytes], T]
    ) -> Input:
        """
        Returns the typed `Input` associated to the provided `port_id`,
        if one is associated, otherwise `None` is returned.

        Args:
            port_id (str): Id associated with the input
            input_type (T): Type of data being received into this input
            deserializer (Callable[[bytes], T]): Deserialization function
                for the given type.

        Returns:
            Input: The typed associated input

        """
        in_stream = self.__inputs.get(port_id, None)
        if in_stream is None:
            return None
        in_stream = Input(in_stream, input_type, deserializer)
        return in_stream

    def take_raw(self, port_id: str) -> RawInput:
        """
        Returns the RawInput associated to the provided `port_id`,
        if one is associated, otherwise `None` is returned.

        A RawInput receives bytes not typed data

        Args:
            port_id (str): Id associated with the input

        Returns:
            RawInput: The raw associated input
        """
        return self.__inputs.get(port_id, None)

    def __repr__(self):
        return self.__str__()

    def __str__(self):
        return f"Inputs(__inputs={self.__inputs})"


class Outputs:
    """
    The `Outputs` structure contains all the sender channels
    we created for a `Source` or an `Operator`.
    """

    def __init__(self, outputs: Dict[str, RawOutput]):
        self.__outputs = outputs

    def take(
        self, port_id: str, output_type: T, serializer: Callable[[T], bytes]
    ) -> Output:
        """

        Returns the typed `Output` associated to the provided `port_id`,
        if one is associated, otherwise `None` is returned.

        Args:
            port_id (str): Id associated with the output
            output_type (T): Type of data being sent to this output
            serializer (Callable[[T], bytes]): Serialization function
                for the given type.

        Returns:
            Output: The typed associated output
        """
        out_stream = self.__outputs.get(port_id, None)
        if out_stream is None:
            return None
        out_stream = Output(out_stream, output_type, serializer)
        return out_stream

    def take_raw(self, port_id: str) -> RawOutput:
        """
        Returns the RawOutput associated to the provided `port_id`,
        if one is associated, otherwise `None` is returned.

        A RawOutput receives bytes not typed data

        Args:
            port_id (str): Id associated with the output

        Returns:
            RawOutput: The raw associated output
        """
        return self.__outputs.get(port_id, None)

    def __repr__(self):
        return self.__str__()

    def __str__(self):
        return f"Outputs(__outputs={self.__outputs})"
