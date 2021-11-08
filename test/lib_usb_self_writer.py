# -*- coding: utf-8 -*-

import protocol_pb2
import select
import threading
import random
import time
import serial

from io import StringIO

from google.protobuf.internal.decoder import _DecodeVarint32
from google.protobuf.internal.encoder import _VarintBytes


class TimeoutError(RuntimeError):
    pass


class self_writer_requestBuilder:
    @staticmethod
    def build_request():
        """
        Создаёт заготовку запроса

        :return: объект типа protocol_pb2.Request c заполнениыми полями id и version
        """
        req = protocol_pb2.Request()
        req.id = random.randrange(0xffffffff)
        req.protocolVersion = protocol_pb2.INFO.PROTOCOL_VERSION
        req.deviceID = protocol_pb2.INFO.ID_DISCOVER
        return req

    @staticmethod
    def build_ping_request():
        """
        Создаёт запрос проверки соединения

        :return: объект типа protocol_pb2.Request
        """
        return self_writer_requestBuilder.build_request()


class self_writer_io:
    """Класс для простого доступа к usb самописцу с использованием google protocol buffers"""
    def __init__(self, port):
        """
        Конструктор

        :param port: Последовательный порт к которому будет произведено подключение
        """
        self.port = port
        self.isConnected = False
    
        self.serial = serial.Serial()
        self.serial.port = self.port
        self.serial.baudrate = 57600
        self.serial.bytesize = serial.EIGHTBITS # number of bits per bytes
        self.serial.parity = serial.PARITY_NONE # set parity check: no parity
        self.serial.stopbits = serial.STOPBITS_ONE # number of stop bits
        self.serial.timeout = 1              # timeout block read
        self.serial.xonxoff = False     # disable software flow control
        self.serial.rtscts = False     # disable hardware (RTS/CTS) flow control
        self.serial.dsrdtr = False       # disable hardware (DSR/DTR) flow control
        self.serial.writeTimeout = 2     # timeout for write

    def __str__(self):
        """
        Выводит краткую информацию о состоянии драйвера

        :return: Строка с краткой информацией ос состоянии устройства и соединения
        """
        return 'Stm32-Self-Writer на порту "{}"'.format(self.port)

    def connect(self):
        """
        Инициирует подключение к устройству

        :return: None
        """
        if self.isConnected:
            raise RuntimeError('Already connected')

        self.serial.open()

        self.isConnected = self.serial.isOpen()

    def disconnect(self):
        """
        Инициирует отключение от устройства

        :return: None
        """
        if not self.isConnected:
            return

        self.serial.close()
        self.isConnected = self.serial.isOpen()

    def process_request_sync(self, request, timeout_sec=1):
        """
        Синхронный обработчик запроса (блокирет вызвавший поток до получения ответа или до истечения таймаута)

        :param request: объект типа protocol_pb2.Request
        :param timeout_sec: Таймаут ожидания ответа от устройства
        :return:
        """
        if not (type(request) is protocol_pb2.Request):
            raise TypeError('"request" mast be instance of "protocol_pb2.Request"')

        return self.process_request_common(request, timeout_sec)

    def process_request_common(self, request, timeout_sec):
        magick = bytes([protocol_pb2.INFO.MAGICK])
        size = _VarintBytes(request.ByteSize())
        body = request.SerializeToString()
        self.serial.write(magick + size + body)

        response = protocol_pb2.Response()

        #self.serial.read(4096)

        #while timeout_sec > 0:
        #    ready = select.select([self.udp_socket], [], [], self.base_timeout)
        #    if ready[0]:
        #        #conn, adr = self.udp_socket.recvfrom(4096)
        #        try:
        #            response.ParseFromString(conn)
        #        except Exception:
        #            continue

        #        if (response.id == request.id) and (adr[0] == self.address)\
        #                and response.protocolVersion <= protocol_pb2.INFO.Value('PROTOCOL_VERSION')\
        #                and response.deviceID == protocol_pb2.INFO.Value('PRESSURE_SELF_WRITER_ID'):
        #            return response  # ok
        #    timeout_sec -= self.base_timeout

        #if timeout_sec <= 0:
        #    raise TimeoutError('Timeout')
        return response

    def async_listener(self, request, callback, timeout_sec):
        try:
            result = self.process_request_common(request, timeout_sec)
        except TimeoutError:
            callback(None)
            return
        callback(result)

    def process_request_async(self, request, callback=None, timeout_sec=1):
        """
        Асинхронный обработчик запроса (вызывает callback, по заверешнию)

        :param request:  объект типа protocol_pb2.Request
        :param callback: функция типа foo(response), которая будет вызвана после получения ответа или истечения таймаута
        :param timeout_sec: Таймаут ожидания ответа от устройства
        :return:
        """
        if not (type(request) is protocol_pb2.Request):
            raise TypeError('"request" mast be instance of "protocol_pb2.Request"')
        self.udp_socket.sendto(request.SerializeToString(), (self.address, self.port))

        thread = threading.Thread(target=self.async_listener, args=(self, request, callback, timeout_sec))
        thread.start()