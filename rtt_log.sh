#!/bin/env bash
echo "######################## RTT 0 ########################"
nc localhost 8765 | defmt-print -e target/thumbv7em-none-eabihf/debug/stm32-usb-self-writer
