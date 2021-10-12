#!/usr/bin/env python

import sys

template = sys.argv[1]
addr_str = sys.argv[2]
outfile = sys.argv[3]

print("-- Generating openocd config --")
print(f"template file: {template};\naddr_str: {addr_str};\noutfile: {outfile}\n")

addr_components = addr_str.split(" ")
if addr_components[-1] != "_SEGGER_RTT":
    print(f"Incorrect symbol: {addr_components[-1]}")
    exit(-1)
else:
    base_addr = "0x" + addr_components[0]
    size = "0x" + addr_components[1]
    print(f"Segger RTT base: {base_addr}, size: {size}")

with open(template) as rf:
    template_data = rf.read()

result = template_data.replace("%RTT_BASE%", base_addr).replace("%RTT_SIZE%", size)

with open(outfile, "w") as wf:
    wf.write(result)
