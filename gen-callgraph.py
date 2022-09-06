#!/usr/bin/env python
# -*- coding: utf-8 -*-

import argparse
import os
import random
import re
import subprocess
import sys
import tempfile

LANG_C_env = os.environ.copy()
LANG_C_env["LANG"] = 'C'


def print_output(text):
    sys.stdout.write(text + '\n')


def print_message(text):
    sys.stderr.write(text + '\n')


def checsk_program_exists(name):
    try:
        subprocess.check_output(["which", name], shell=False)
    except:
        return False
    return True


def gen_sym_table(file, readelf):
    return subprocess.Popen([readelf, '--headers', '--symbols', file],
        stdout=subprocess.PIPE, env=LANG_C_env).communicate()[0]


def gen_asm_file(file, objdump):
    return subprocess.Popen([objdump, '-d', file], stdout=subprocess.PIPE,
        env=LANG_C_env).communicate()[0]


def mktemp():
    return tempfile.NamedTemporaryFile(dir='/tmp', delete=False)


def grep(lines, fragment):
    res = []
    for line in lines:
        if fragment in line:
            res.append(line)

    return res


def parce_hex(string):
    return int(string, 16)


def get_entry_point_address(sym_table):
    entry_point_line = grep(sym_table, 'Entry point address:')
    return parce_hex(re.search('0x([0-9A-Fa-f]+)', entry_point_line[0]).group(0))


def generate_function_dict(sym_table):
    function_search_pattern= re.compile('.*: ([0-9A-Fa-f]+) *([0-9]+) *([A-Z]+) *([A-Z]+) *([A-Z]+) *[0-9A-Z]+ *(.*)')

    func2addr_dict = {}
    found_symtab = False

    for line in sym_table:
        if not found_symtab:
            found_symtab = 'Symbol table \'.symtab\'' in line

        #print(line)
        match = function_search_pattern.search(line)
        if not (match is None):
            #if match.group(3) == 'FUNC' and match.group(4) == 'GLOBAL' and match.group(3) != 'UND':
            if match.group(3) == 'FUNC' and match.group(3) != 'UND':
                addr = parce_hex(match.group(1))
                addr -= 1  # почему-то там все адреса +1
                func2addr_dict[addr] = match.group(6)

    if not found_symtab:
        raise RuntimeError('Таблица символов .symtab не найдена')

    return func2addr_dict


def cpp_filt(cppfilt, fname):
    return subprocess.Popen([cppfilt, fname], stdout=subprocess.PIPE,
        env=LANG_C_env).communicate()[0].decode('utf-8').strip()


def generete_nodes(symtable, cppfilt, entry_addr):
    res = []
    for addr, fname in symtable.items():
        func_name_demangled = cpp_filt(cppfilt, fname)
        shape_spec = ''
        if addr == entry_addr:
            shape_spec = ', shape="box"'
        res.append('"{}" [label="0x{:08X}: {}"{}];'.format(fname, addr, func_name_demangled, shape_spec))

    return res


def gen_random_color():
    return '{:3f} {:3f} {:3f}'.format(random.random(), random.random(), random.random())


def generate_edges(symtable, disassembly_list, call_cmd):
    sorted_fun_adress_list = sorted(symtable.keys())
    parce_command_re = re.compile('{}.+[\t ]+([0-9a-f]+) <'.format(call_cmd))

    # disassembly to dict
    disasm_dict = {}
    #  8000704:       f105 0034       add.w   r0, r5, #52     ; 0x34
    disasm_parce_template = re.compile('^([ 0-9a-f]+):[\t ]+[0-9a-f ]+[\t ]+(.*)$')

    for line in disassembly_list:
        _mach = disasm_parce_template.search(line)
        if _mach:
            addr = parce_hex(_mach.group(1))
            disasm_dict[addr] = _mach.group(2)

    for i in range(len(sorted_fun_adress_list)):
        start = sorted_fun_adress_list[i]
        this_func = symtable[start]
        try:
            stop = sorted_fun_adress_list[i + 1]
        except:
            stop = sorted(disasm_dict.keys())[-1]

        for addr in range(start, stop + 1):
            try:
                command = disasm_dict[addr]
            except:
                continue
            mach = parce_command_re.search(command)

            if mach:
                call_addr = parce_hex(mach.group(1))
                try:
                    call_fun = symtable[call_addr]
                except KeyError:
                    continue

                call = '"{}" -> "{}" [label="0x{:08X}" color="{}"];'.format(
                    this_func, call_fun, addr, gen_random_color())

                yield call


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--toolprefix", type=str, help="префикс тулчейна", default='')
    parser.add_argument("--call_cmd", type=str, help="Ассемблерная инструкция вызова функции", default='callq')
    parser.add_argument("--debug", action='store_true', help='Сохранять промежуточные данные во временные файлы' , default=False)
    parser.add_argument("elf",type=str, help="Имя обрабатываемого файла")

    args = parser.parse_args()

    READELF = args.toolprefix + 'readelf'
    OBJDUMP = args.toolprefix + 'objdump'
    CPPFILT = args.toolprefix + 'c++filt'
    DOT     = 'dot'

    for prg in (READELF, OBJDUMP, CPPFILT):
        if not checsk_program_exists(prg):
            print_message('Error: Requires {} in $PATH'.format(prg))
            quit(1)

    if not checsk_program_exists(DOT):
        print_message('Error: Requires dot in $PATH for {} {} | dot -Tsvg -ocallgraph.svg'.format(sys.argv[0], args.elf))

    sym_file_contents = gen_sym_table(args.elf, READELF)
    asm_file_contents =  gen_asm_file(args.elf, OBJDUMP)

    if args.debug:
        sym_dbg_file = mktemp()
        sym_dbg_file.write(sym_file_contents)
        print_message('Symbol table writen to {}'.format(sym_dbg_file.name))
        asm_dbg_file = mktemp()
        asm_dbg_file.write(asm_file_contents)
        print_message('Disaembly writen to {}'.format(asm_dbg_file.name))

    sym_file_contents = sym_file_contents.decode('utf-8').split('\n')
    asm_file_contents = asm_file_contents.decode('utf-8').split('\n')

    entry_point_addr = get_entry_point_address(sym_file_contents)

    print_message('Generating function address pairs.. (Step 1 of 3)')

    symtable = generate_function_dict(sym_file_contents)

    if args.debug:
        symtable_dbg_file = mktemp()
        table_list = []
        for key, value in symtable.items():
            table_list.append('{:08X} {}'.format(key, value))
        symtable_dbg_file.write(('\n'.join(sorted(table_list))).encode('utf-8'))
        print_message('Symbol dict writen to {}'.format(symtable_dbg_file.name))

    print_message('Generating nodes.. (Step 2 of 3)')

    print_output('digraph "{}" {{'.format(args.elf.split('/')[-1]))
    print_output('rankdir=LR;')
    print_output('node [shape=ellipse];')

    for node in generete_nodes(symtable, CPPFILT, entry_point_addr):
        print_output(node)

    print_message('Generating edges.. (Step 3 of 3)')

    for edge in generate_edges(symtable, asm_file_contents, args.call_cmd):
        print_output(edge)

    print_output('}')


# чтобы при импорте не выполнялся код автоматом
if __name__ == '__main__':
    main()
