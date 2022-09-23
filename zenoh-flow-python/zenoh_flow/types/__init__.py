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

# from uuid import UUID
from enum import Enum
from typing import Sequence, Optional

# class Context(object):
#     '''
#         A Zenoh Flow context.
#         Zenoh Flow context contains a `mode` that represent
#         the current execution mode for the operator.
#     '''
#     def __init__(self):
#         self.async_callbacks = {}
#         self.sync_callbacks = {}

#     def register_async_callback(self, input_recv, cb):
#         input_recv.disable_recv()
#         self.async_callbacks[port_id] = cb

#     def register_callback(self, port_id, cb):
#         self.sync_callbacks[port_id] = cb


class Timestamp(object):
    """
    The Zenoh (Flow) timestamp.
    """

    def __init__(self, ntp: int, id: str):
        self.ntp = ntp
        self.id = id
