##
## Copyright (c) 2022 ZettaScale Technology
##
## This program and the accompanying materials are made available under the
## terms of the Eclipse Public License 2.0 which is available at
## http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
## which is available at https://www.apache.org/licenses/LICENSE-2.0.
##
## SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
##
## Contributors:
##   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
##

from zenoh_flow import Inputs, Outputs, Source
import time

class MyState:
    def __init__(self, configuration):
        self.value = 0
        if configuration['value'] is not None:
            self.value = int(configuration['value'])

class MySrc(Source):
    def initialize(self, configuration):
        return MyState(configuration)

    def finalize(self, state):
        return None

    def run(self, _ctx, state):
        state.value += 1
        time.sleep(1)
        return int_to_bytes(state.value)



def int_to_bytes(x: int) -> bytes:
    return x.to_bytes((x.bit_length() + 7) // 8, 'big')

def register():
    return MySrc
