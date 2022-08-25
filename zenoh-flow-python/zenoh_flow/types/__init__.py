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
#     def __init__(self, mode: int):
#         self.mode = mode

#     def get_mode(self) -> int:
#         '''
#             Gets the mode from the :class:`Context`

#             :rtype: int
#         '''
#         return self.mode

#     def set_mode(self, mode: int) -> None:
#         '''
#             Sets the mode for the :class:`Context`

#             :param mode the mode to be set
#         '''
#         self.mode = mode


class Timestamp(object):
    """
    The Zenoh (Flow) timestamp.
    """

    def __init__(self, ntp: int, id: str):
        self.ntp = ntp
        self.id = id


# class DataMessage(object):
#     '''
#         A Zenoh Flow Data Message.
#         It contains:
#         `data` as array of bytes.
#         `ts` an uHLC timestamp associated with the data.
#     '''

#     def __init__(self, ts : Timestamp, data: bytes):
#         self.ts = ts
#         self.data = data

#     def get_data(self) -> bytes:
#         '''
#             Gets the data from the :class:`DataMessage`

#             :rtype: bytes
#         '''
#         return self.data

#     def get_timestamp(self) -> Timestamp:
#         '''
#             Gets the timestamp from the :class:`DataMessage`

#             :rtype: bytes
#         '''
#         return self.ts


# class Watermark(object):
#     '''
#         A Zenoh Flow Watermark Message.
#         It contains:
#         `ts` an uHLC timestamp watermaks.
#     '''
#     def __init__(self, ts : Timestamp):
#         self.data = data

#     def get_timestamp(self) -> Timestamp:
#         '''
#             Gets the timestamp from the :class:`Watermark`

#             :rtype: bytes
#         '''
#         return self.ts
