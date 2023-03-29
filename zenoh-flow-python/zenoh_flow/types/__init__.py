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


from zenoh_flow import InnerInput, InnerOutput
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
        self,
        runtime_name: str,
        runtime_uuid: str,
        flow_name: str,
        instance_uuid: str
    ):
        self.runtime_name = runtime_name
        """Name of the runtime where the node is running."""
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
            f"Context(runtime_name={self.runtime_name}, "
            + f"runtime_uuid={self.runtime_uuid}, "
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


class DataMessage:
    """
    Zenoh Flow data messages
    It contains the actual data, the timestamp associated, and
    information whether the message is a `Watermark`
    """
    def __init__(self, data: Any, ts: int,  watermark: bool):
        self.__data = data
        self.__ts = ts
        self.__watermark = watermark

    def get_data(self) -> Any:
        """
        Returns a reference over bytes representing the data.
        """
        return self.__data

    def get_ts(self) -> int:
        """
        Returns the data timestamp.
        """
        return self.__ts

    def is_watermark(self) -> bool:
        """
        Returns whether the `DataMessage` is a watermark or not.
        """
        return self.__watermark


class Input:
    """
    Channel that receives data from upstream nodes.
    """
    def __init__(self, inner: InnerInput, input_type: T, deserializer: Callable[[bytes], T]):
        self.__deserializer = deserializer
        self.__inner = inner
        self.__type = input_type

    async def recv(self) -> DataMessage:
        """
        Returns the first `DataMessage` that was received, *asynchronously*,
        on any of the channels associated with this Input.

        If several `DataMessage` are received at the same time,
        one is randomly selected.
        """
        data_msg = await self.__inner.recv()
        data = None
        if len(data_msg.data) > 0:
            data = self.__deserializer(data_msg.data)
        msg = DataMessage(data, data_msg.ts, data_msg.is_watermark)
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
    def __init__(self, inner: InnerOutput, output_type: T, serializer: Callable[[T], bytes]):
        self.__serializer = serializer
        self.__inner = inner
        self.__type = output_type

    async def send(self, data: T, ts: Optional[int] = None):
        """_summary_
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
    def __init__(self, inputs: Dict[str, InnerInput]):
        self.__inputs = inputs

    def take(self, port_id: str, input_type: T,  deserializer: Callable[[bytes], T]) -> Input:
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
        in_stream = self.__input.get(port_id, None)
        if in_stream is None:
            return None
        in_stream = Input(in_stream, input_type, deserializer)


class Outputs:
    """
    The `Outputs` structure contains all the sender channels
    we created for a `Source` or an `Operator`.
    """
    def __init__(self, outputs: Dict[str, InnerOutput]):
        self.__outputs = outputs

    def take(self, port_id: str, output_type: T,  serializer: Callable[[T], bytes]) -> Output:
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

