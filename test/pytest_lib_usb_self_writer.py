#!/usr/bin/env python
# -*- coding: utf-8 -*-

import os
import sys
import pytest
import lib_usb_self_writer as lib
import protocol_pb2


def print_help():
    print(f"""Возможно вы пытаитесь запустить этот файл напрямую?\n
Запустите: $ SERIAL=/dev/tty<порт> py.test -q {sys.argv[0]}\n""")


@pytest.fixture
def device(request):
    if not ('SERIAL' in os.environ.keys()):
        print("Не указан порт для теста!\n")
        print_help()
        assert 0

    d = lib.self_writer_io(os.environ['SERIAL'])
    d.connect()

    def fin():
        d.disconnect()

    request.addfinalizer(fin)
    return d


@pytest.fixture
def settime_req():
    return lib.self_writer_requestBuilder.build_set_time_request()


@pytest.fixture
def settings_req():
    return lib.self_writer_requestBuilder.build_settings_request()


def test_ping(device):
    req = lib.self_writer_requestBuilder.build_ping_request()
    resp = device.process_request_sync(req)
    assert resp
    assert resp.Global_status == protocol_pb2.STATUS.OK

def test_read_settings(device, settings_req):
    resp = device.process_request_sync(settings_req)
    assert resp
    assert resp.getSettings
    assert resp.Global_status == protocol_pb2.STATUS.OK