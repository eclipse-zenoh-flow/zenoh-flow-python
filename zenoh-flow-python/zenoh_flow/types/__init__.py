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


from zenoh_flow import DataReceiver, DataSender, DataMessage
from typing import Dict, Any
from typing import Callable, Awaitable


class Context(object):
    """
    A Zenoh Flow context.
    Zenoh Flow context provides access to runtime and flow information to
    the operator.

    Attributes:
        runtime_name    Name of the runtime where the node is running.
        runtime_uuid    UUID of the runtime where the node is running.
        flow_name       Flow of which the node is part.
        instance_uuid   UUID of the flow instance the node is associated.

    """

    def __init__(
        self, runtime_name: str, runtime_uuid: str, flow_name: str, instance_uuid: str
    ):
        self.runtime_name = runtime_name
        self.runtime_uuid = runtime_uuid
        self.flow_name = flow_name
        self.instance_uuid = instance_uuid

    def __repr__(self):
        return self.__str__()

    def __str__(self):
        return (
            f"Context(runtime_name={self.runtime_name}, "
            + f"runtime_uuid={self.runtime_uuid}, "
            + f"flow_name={self.flow_name}, instance_uuid={self.instance_uuid})"
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
        return f"Timestamp(ntp={self.ntp}, " + f"id={self.id})"
