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



class Sink(object):

    def run(self, context, state, input: bytes) -> None:
        NotImplementedError("Please implement your own method, Sink is an interface")

    def initialize(self, configuration):
        NotImplementedError("Please implement your own method, Sink is an interface")

    def finalize(self, state) -> None:
        NotImplementedError("Please implement your own method, Sink is an interface")