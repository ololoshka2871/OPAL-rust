#!/usr/bin/env python
# -*- coding: utf-8 -*-


import argparse
import os
import sys
import time
import libr4_24_2
import protocol_pb2


def print_hader(stream, chanel_mask):
    stream.write('Время[мс];Время обработки запроса[мс];')
    for i in range(32):
        if chanel_mask & (1 << i):
            stream.write('Канал {0} измерен в [мc];Канал {0} частота [Гц];'.format(i))

    stream.write('\n')


def gen_pattern(chanel_mask):
    res = '{0};{1};'
    c = 2
    for i in range(32):
        if chanel_mask & (1 << i):
            res += ('{fn[0]};{fn[1]};'.replace('fn', str(c)))
            c += 1

    return res


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--interval', '-i', type=float, help="Интервал опроса [c]", default=0)
    parser.add_argument('--chanel_mask', '-c', type=int, help="Маска каналов для опроса (-1 - все)", default=1)
    parser.add_argument('--measure_time', '-m', type=int, help="Время измерения [мс]", default=10)
    parser.add_argument('--port', '-p', type=int, help='UDP port', default=9128)

    args = parser.parse_args()
    if not ('TEST_IP' in os.environ.keys()):
        print("Не указан IP адрес для теста!\n")
        print("Возможно вы пытаитесь запустить этот файл напрямую?\n"
              "Запстите: $ TEST_IP=<ip_аддресс> make pb_reader.run\n")
        return 1

    if args.chanel_mask < 0:
        args.chanel_mask = (1 << 24) - 1

    ch2rd = []
    for i in range(32):
        if args.chanel_mask & (1 << i):
            ch2rd.append(i)

    request = libr4_24_2.r4_24_2_requestBuilder.build_getmeasureresults_request(ch2rd)
    device = libr4_24_2.r4_24_2_io(os.environ['TEST_IP'])
    device.connect()

    device.enable_channels(ch2rd)
    device.setMeasureTime(ch2rd, args.measure_time)
    #device.setClock(time.time())

    time.sleep(2 * args.measure_time / 1000.0)

    print_hader(sys.stdout, args.chanel_mask)
    pattern = gen_pattern(args.chanel_mask)

    while True:
        start = time.time()
        response = device.process_request_sync(request)
        req_time = time.time()
        if (not response) or response.Global_status != protocol_pb2.STATUS.Value('OK'):
            raise RuntimeError('Error {} during read values'.format(response.Global_status))

        results = {}
        for i in response.getMeasureResultsResponce.results._values:
            results[i.chanelNumber] = (i.timestamp, i.Frequency)

        res_list = []
        for key in sorted(results.keys()):
            res_list.append(results[key])

        pr_time = time.time()
        print(pattern.format(long(pr_time * 1000),
                             long((req_time - start) * 1000),
                             *res_list))

        # sleep if needed
        processed = time.time() - start
        if processed < args.interval:
            time.sleep(args.interval - processed)

# чтобы при импорте не выполнялся код автоматом
if __name__ == '__main__':
    main()
