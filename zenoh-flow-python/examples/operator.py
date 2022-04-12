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

from zenoh_flow.interfaces import Operator

class MyState:
    def __init__(self):
        self.value = 0

    def inc(self):
        self.value += 1

    def mod_2(self):
        return (self.value % 2)

    def mod_3(self):
        return (self.value % 3)

class MyOp(Operator):
    def initialize(self, configuration):
        return MyState()

    def finalize(self, state):
        return None

    def input_rule(self, _ctx, state, tokens):
        # Using input rules
        state.inc()
        token = tokens.get('Data')
        if state.mod_2():
            token.set_action_consume()
            return True
        elif state.mod_3():
            token.set_action_keep()
            return True
        else:
            token.set_action_drop()
            return False

    def output_rule(self, _ctx, _state, outputs, _deadline_miss = None):
        return outputs

    def run(self, _ctx, _state, inputs):
        # Getting the inputs
        data = inputs.get('Data').get_data()
        # Computing over the inputs
        int_data = int_from_bytes(data)
        int_data = int_data * 2
        # outputs = ()
        # outputs.put('Data', int_to_bytes(int_data))
        # Producing the outputs
        outputs = {'Data' : int_to_bytes(int_data)}
        return outputs



def int_to_bytes(x: int) -> bytes:
    return x.to_bytes((x.bit_length() + 7) // 8, 'big')

def int_from_bytes(xbytes: bytes) -> int:
    return int.from_bytes(xbytes, 'big')

def register():
    return MyOp