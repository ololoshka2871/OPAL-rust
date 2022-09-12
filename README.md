# README

# [cargo make](https://sagiegurari.github.io/cargo-make/)
1. flash - use openocd
2. log - defmt log, stagt debuginf first!

# Connection

## GALVO
* CLOCK - PB3 
* SYNC - PB4
* CHAIN[1..2] - PB[5..6]

## Laser
* D[0..7] - PA[0..7] Паралельная шина
* LATCH - PA9 Защелка (возможно не нужна)
* ALARM[1..3] - PC[13..15] Статус лазера, там еще есть нулевой бит, но он не используется и не подключен
* LSYNC - PB7 (TIM4_CH2) - меандр 50% с частотой указаной в паспорте на лазер
* EM - PB8 (TIM4_CH3) - модуляция лазера, предположительно шим синхронный с LSYNC, но с произвольным заполнением
* EE - PB9 (TIM4_CH4) - предположительно просто разрешает стрелять
* RED_LASER - PA10 (TIM1_CH3) - просто включает красный указательный лезер, возможно можно шимить его.
* USB_PULL_UP - PA15 - включает подтяжку D+

## Распределение переферии
* TIM4 (CH2, CH3, CH4?) - PWM - Лазер
* TIM1 (CH3) - Красный лазер
* TIM2 - триггер для DMA
* DMA1 - TIM2_UP (CHANNEL2) - Копирует из буфера в регистр GPIOB -> GALVO
