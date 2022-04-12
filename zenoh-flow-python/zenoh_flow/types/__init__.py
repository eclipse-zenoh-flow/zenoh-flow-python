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

class Context(object):
    '''
        A Zenoh Flow context.
        Zenoh Flow context contains a `mode` that represent
        the current execution mode for the operator.
    '''
    def __init__(self, mode: int):
        self.mode = mode

    def get_mode(self) -> int:
        '''
            Gets the mode from the :class:`Context`

            :rtype: int
        '''
        return self.mode

    def set_mode(self, mode: int) -> None:
        '''
            Sets the mode for the :class:`Context`

            :param mode the mode to be set
        '''
        self.mode = mode



class FromDescriptor(object):
    '''
        The descriptor on where an E2E Deadline starts.
    '''
    def __init__(self, node: str, output: str):
        self.node = node
        self.output = output



class ToDescriptor(object):
    '''
        The descriptor on where a E2E Deadline ends.
    '''
    def __init__(self, node: str, input: str):
        self.node = node
        self.input = input

class E2EDeadlineMiss(object):
    '''
        A End to End Deadline.
        A deadline can apply for a whole graph or for a subpart of it.
    '''
    def __init__(self, frm: FromDescriptor, to: ToDescriptor, start: int, end: int):
        self.frm = frm
        self.to = to
        self.start = start
        self.stop = stop

class LocalDeadlineMiss(object):
    '''
        A Zenoh Flow local deadline miss.
        A structure containing all the information regarding a missed, local, deadline.
        A local deadline is represented by a maximum time between receiving the
        data at the Input Rules and providing a result to the Output Rule.
        This means that if the Run function takes more that the deadline
        the Output Rule will be notified by the means of this
        `LocalDeadlineMiss`.
    '''

    def __init__(self, deadline: int, elapsed: int):
        self.deadline = deadline
        self.elapsed = elapsed


class Timestamp(object):
    '''
        The Zenoh (Flow) timestamp.
    '''
    def __init__(self, ntp: int, id: str):
        self.ntp = ntp
        self.id = id



class TokenAction(Enum):
    '''
        The Action that can be taken on a token.
    '''

    Drop = 0
    Consume = 1
    Keep = 2

class TokenStatus(Enum):
    '''
        The status of a token.
    '''
    Ready = 0
    Pending = 1


class InputToken(object):
    '''
        A Zenoh Flow Input Token
    '''
    def __init__(self, status: TokenStatus,  action: Optional[TokenAction] = None, data: Optional[bytes] = None, timestamp: Optional[Timestamp] = None):
        self.status = status
        self.action = action
        self.data = data
        self.e2d_deadlines = []
        self.timestamp = timestamp

    def set_action_drop(self) -> None:
        '''
            Sets the token to be dropped.
        '''
        if self.is_ready():
            self.action = TokenAction.Drop
        return None

    def set_action_consume(self) -> None:
        '''
            Sets the token to be consumed in the current iteration (default).
        '''
        if self.is_ready():
            self.action = TokenAction.Consume
        return None

    def set_action_keep(self) -> None:
        '''
            Sets the token to be kept for next iteration.
        '''
        if self.is_ready():
            self.action = TokenAction.Keep
        return None

    def get_action(self) -> TokenAction:
        '''
            Gets the action from the :class:`InputToken`

            :rtype: str
        '''

        return self.action

    def is_ready(self) -> bool:
        '''
            Checks if the :class:`InputToken` is ready.
            i.e. has Data.

            :rtype: bool
        '''
        return self.status == TokenStatus.Ready

    def is_pending(self) -> bool:
        '''
            Checks if the :class:`InputToken` is pending.
            i.e. has no data.

            :rtype: bool
        '''
        return self.status == TokenStatus.Pending

    def get_data(self) -> bytes:
        '''
            Gets the data from the :class:`InputToken`

            :rtype: bytes
        '''
        return self.data

    def get_timestamp(self) -> Timestamp:
        '''
            Gets the timestamp from the :class:`InputToken`

            :rtype: bytes
        '''
        return self.timestamp



class DataMessage(object):
    '''
        A Zenoh Flow Data Message.
        It contains:
        `data` as array of bytes.
        `ts` an uHLC timestamp associated with the data.
        `missed_end_to_end_deadlines` list of `E2EDeadlineMiss`
    '''

    def __init__(self, ts : Timestamp, data: bytes, missed_end_to_end_deadlines: Optional[Sequence[E2EDeadlineMiss]] = []):
        self.ts = ts
        self.data = data
        self.missed_end_to_end_deadlines = missed_end_to_end_deadlines

    def get_data(self) -> bytes:
        '''
            Gets the data from the :class:`DataMessage`

            :rtype: bytes
        '''
        return self.data

    def get_timestamp(self) -> Timestamp:
        '''
            Gets the timestamp from the :class:`DataMessage`

            :rtype: bytes
        '''
        return self.ts


