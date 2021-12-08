use core::{fmt::Debug, marker::PhantomData};

use alloc::sync::Arc;
use freertos_rust::{Duration, InterruptContext};
use stm32l4xx_hal::{gpio::State, prelude::OutputPin};

use crate::{
    sensors::freqmeter::{
        master_counter::{MasterCounter, MasterTimerInfo},
        InCounter, OnCycleFinished,
    },
    support::interrupt_controller::{IInterruptController, Interrupt},
};

pub struct SensorPerith<TIM1, DMA1, TIM2, DMA2, PIN1, PIN2, ENPIN1, ENPIN2>
// Суть в том, что мы напишем КОНКРЕТНУЮ имплементацию InCounter<DMA> для
// конкретного счетчика рандомная пара не соберется.
where
    TIM1: InCounter<DMA1, PIN1>,
    TIM2: InCounter<DMA2, PIN2>,
    ENPIN1: ,
{
    pub timer1: TIM1,
    pub timer1_dma_ch: DMA1,
    pub timer1_pin: PIN1,
    pub en_1: ENPIN1,

    pub timer2: TIM2,
    pub timer2_dma_ch: DMA2,
    pub timer2_pin: PIN2,
    pub en_2: ENPIN2,
}

#[derive(Clone, Copy, Debug, PartialEq, defmt::Format)]
pub enum Channel {
    Pressure = 0,
    Temperature = 1,
    TCPU = 2,
    Vbat = 3,
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    Start(Channel),
    Stop(Channel),
    Ready(Channel, u32, u32, bool),
}

trait Processor {
    fn enable(&mut self);
    fn diasbe(&mut self);

    fn is_initial_result(&mut self) -> bool;
}

struct FreqmeterEnable<'a, TIM, DMA, INPIN, ENPIN>
where
    TIM: InCounter<DMA, INPIN>,
{
    freqmeter: &'a mut TIM,
    gpio_pin: ENPIN,
    startup: bool,
    _phantomdata1: PhantomData<DMA>,
    _phantomdata2: PhantomData<INPIN>,
}

struct DMAFinished {
    master: MasterTimerInfo,
    cc: Arc<freertos_rust::Queue<Command>>,
    ic: Arc<dyn IInterruptController>,
    ch: Channel,
}

impl<'a, TIM, DMA, INPIN, ENPIN> Processor for FreqmeterEnable<'a, TIM, DMA, INPIN, ENPIN>
where
    TIM: InCounter<DMA, INPIN>,
    ENPIN: OutputPin,
    <ENPIN as OutputPin>::Error: Debug,
{
    fn enable(&mut self) {
        self.set_lvl(crate::config::GENERATOR_ENABLE_LVL);
        self.startup = true;
        self.freqmeter.cold_start();
    }

    fn diasbe(&mut self) {
        self.freqmeter.stop();
        self.set_lvl(crate::config::GENERATOR_DISABLE_LVL);
    }

    fn is_initial_result(&mut self) -> bool {
        if self.startup {
            self.startup = false;
            true
        } else {
            false
        }
    }
}

impl<'a, TIM, DMA, INPIN, ENPIN> FreqmeterEnable<'a, TIM, DMA, INPIN, ENPIN>
where
    TIM: InCounter<DMA, INPIN>,
    ENPIN: OutputPin,
    <ENPIN as OutputPin>::Error: Debug,
{
    fn new(freqmeter: &'a mut TIM, gpio_pin: ENPIN) -> Self {
        Self {
            freqmeter,
            gpio_pin,
            startup: false,
            _phantomdata1: PhantomData,
            _phantomdata2: PhantomData,
        }
    }

    fn set_lvl(&mut self, lvl: State) {
        match lvl {
            State::High => self.gpio_pin.set_high().unwrap(),
            State::Low => self.gpio_pin.set_low().unwrap(),
        }
    }
}

impl DMAFinished {
    fn new(
        master: MasterTimerInfo,
        cc: Arc<freertos_rust::Queue<Command>>,
        ic: Arc<dyn IInterruptController>,
        ch: Channel,
    ) -> Self {
        Self { master, cc, ic, ch }
    }
}

impl OnCycleFinished for DMAFinished {
    fn cycle_finished(&self, captured: u32, target: u32, irq: Interrupt) {
        let mut ctx = InterruptContext::new();
        let (result, wraped) = self.master.update_captured_value(captured);
        if let Err(e) = self
            .cc
            .send_from_isr(&mut ctx, Command::Ready(self.ch, target, result, wraped))
        {
            defmt::error!("Sensor command queue error: {}", defmt::Debug2Format(&e));
        }
        self.ic.unpend(irq);
    }
}

pub fn sensor_processor<PTIM, PDMA, TTIM, TDMA, PPIN, TPIN, ENPIN1, ENPIN2>(
    mut perith: SensorPerith<PTIM, PDMA, TTIM, TDMA, PPIN, TPIN, ENPIN1, ENPIN2>,
    command_queue: Arc<freertos_rust::Queue<Command>>,
    ic: Arc<dyn IInterruptController>,
) -> !
where
    PTIM: InCounter<PDMA, PPIN>,
    TTIM: InCounter<TDMA, TPIN>,
    ENPIN1: OutputPin,
    <ENPIN1 as OutputPin>::Error: Debug,
    ENPIN2: OutputPin,
    <ENPIN2 as OutputPin>::Error: Debug,
{
    let master_counter = MasterCounter::allocate().unwrap();
    perith.timer1.configure(
        master_counter.cnt_addr(),
        &mut perith.timer1_dma_ch,
        perith.timer1_pin,
        ic.as_ref(),
        DMAFinished::new(
            master_counter,
            command_queue.clone(),
            ic.clone(),
            Channel::Pressure,
        ),
    );

    let master_counter = MasterCounter::allocate().unwrap();
    perith.timer2.configure(
        master_counter.cnt_addr(),
        &mut perith.timer2_dma_ch,
        perith.timer2_pin,
        ic.as_ref(),
        DMAFinished::new(
            master_counter,
            command_queue.clone(),
            ic.clone(),
            Channel::Temperature,
        ),
    );

    let mut enabler1 = FreqmeterEnable::new(&mut perith.timer1, perith.en_1);
    let mut enabler2 = FreqmeterEnable::new(&mut perith.timer2, perith.en_2);

    let mut channels: [&mut dyn Processor; 2] = [&mut enabler1, &mut enabler2];

    //------------------ remove after tests --------------
    crate::settings::settings_action::<_, _, _, ()>(Duration::infinite(), |(ws, _)| {
        let flags = [ws.P_enabled, ws.T_enabled, ws.TCPUEnabled, ws.VBatEnabled];
        for (c, f) in channels.iter_mut().zip(flags.iter()) {
            if *f {
                (*c).enable();
            }
        }

        Ok(())
    })
    .expect("Failed to read channel enable");
    //-----------------------------------------------------

    let mut prev = [0u32; 2];

    loop {
        if let Ok(cmd) = command_queue.receive(Duration::zero()) {
            match cmd {
                Command::Start(c) => {
                    channels[c as usize].enable();
                }
                Command::Stop(c) => {
                    channels[c as usize].diasbe();
                }
                Command::Ready(c, target, result, wraped) => {
                    let prev = &mut prev[c as usize];

                    if channels[c as usize].is_initial_result() {
                        *prev = result;
                    } else {
                        let diff = if *prev <= result {
                            result - *prev
                        } else {
                            u32::MAX - *prev + result
                        } as f32;

                        *prev = result;

                        let f = 20000000.0f32 / diff * target as f32;

                        if wraped {
                            defmt::warn!(
                                "Sensor result: c={}, target={}, diff={}, wraped={}: F = {}",
                                c,
                                target,
                                diff,
                                wraped,
                                f
                            );
                        } else {
                            defmt::trace!(
                                "Sensor result: c={}, target={}, diff={}, wraped={}: F = {}",
                                c,
                                target,
                                diff,
                                wraped,
                                f
                            );
                        }
                    }
                }
            }
        }
    }
}
