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

from uuid import UUID
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
    def __init__(self, node: str, output: str):
        self.node = node
        self.output = output



class ToDescriptor(object):
    def __init__(self, node: str, input: str):
        self.node = node
        self.input = input

class E2EDeadlineMiss(object):
    def __init__(self, frm: FromDescriptor, to: ToDescriptor, start: int, end: int):
        self.frm = frm
        self.to = to
        self.start = start
        self.stop = stop

class LocalDeadlineMiss(object):
    def __init__(self, deadline: int, elapsed: int):
        self.deadline = deadline
        self.elapsed = elapsed


class Timestamp(object):
    def __init__(self, ntp: int, id: UUID):
        self.ntp = ntp
        self.id = id



class TokenAction(Enum):
    Drop = 0
    Consume = 1
    Keep = 2

class TokenStatus(Enum):
    Ready = 0
    Pending = 1


class InputToken(object):
    def __init__(self, status: TokenStatus,  action: Optional[TokenAction] = None, data: Optional[bytes] = None):
        self.status = status
        self.action = action
        self.data = data

    def set_action_drop(self) -> None:
        if self.is_ready():
            self.action = TokenAction.Drop
        return None

    def set_action_consume(self) -> None:
        if self.is_ready():
            self.action = TokenAction.Consume
        return None

    def set_action_keep(self) -> None:
        if self.is_ready():
            self.action = TokenAction.Keep
        return None

    def get_action(self) -> TokenAction:
        return self.action

    def is_ready(self) -> bool:
        return self.status == TokenStatus.Ready

    def is_pending(self) -> bool:
        return self.status == TokenStatus.Pending

    def get_data(self) -> bytes:
        return self.data


class DataMessage(object):
    def __init__(ts : Timestamp, data: bytes, missed_end_to_end_deadlines: Sequence[E2EDeadlineMiss] ):
        self.ts = ts
        self.data = data
        self.missed_end_to_end_deadlines = missed_end_to_end_deadlines

