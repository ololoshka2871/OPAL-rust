use core::fmt::Debug;

use alloc::sync::Arc;
use freertos_rust::{Duration, InterruptContext};
use stm32l4xx_hal::prelude::OutputPin;
use strum::IntoStaticStr;

use crate::{
    sensors::freqmeter::{
        master_counter::{MasterCounter, MasterTimerInfo},
        FChProcessor, FreqmeterController, InCounter, OnCycleFinished, TimerEvent,
    },
    support::interrupt_controller::{IInterruptController, Interrupt},
    workmodes::processing::RawValueProcessor,
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

#[derive(Clone, Copy, Debug, PartialEq, defmt::Format, IntoStaticStr)]
pub enum FChannel {
    Pressure = 0,
    Temperature = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, defmt::Format)]
pub enum AChannel {
    TCPU = 0,
    Vbat = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, defmt::Format)]
pub enum Channel {
    FChannel(FChannel),
    AChannel(AChannel),
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    Start(Channel),
    Stop(Channel),
    ReadyFChannel(FChannel, TimerEvent, u32, u32, bool),
    ReadyAChannel(AChannel),
    SetTarget(FChannel, u32, u32),
    TimeoutFChannel(FChannel),
}

struct DMAFinished {
    master: MasterTimerInfo,
    cc: Arc<freertos_rust::Queue<Command>>,
    ic: Arc<dyn IInterruptController>,
    ch: FChannel,
}

impl DMAFinished {
    fn new(
        master: MasterTimerInfo,
        cc: Arc<freertos_rust::Queue<Command>>,
        ic: Arc<dyn IInterruptController>,
        ch: FChannel,
    ) -> Self {
        Self { master, cc, ic, ch }
    }
}

impl OnCycleFinished for DMAFinished {
    fn cycle_finished(&self, event: TimerEvent, captured: u32, target: u32, irq: Interrupt) {
        let mut ctx = InterruptContext::new();
        let (result, wraped) = self.master.update_captured_value(captured);
        if let Err(e) = self.cc.send_from_isr(
            &mut ctx,
            Command::ReadyFChannel(self.ch, event, target, result, wraped),
        ) {
            defmt::error!("Sensor command queue error: {}", defmt::Debug2Format(&e));
        }
        self.ic.unpend(irq);
    }
}

pub fn sensor_processor<PTIM, PDMA, TTIM, TDMA, PPIN, TPIN, ENPIN1, ENPIN2, TP>(
    mut perith: SensorPerith<PTIM, PDMA, TTIM, TDMA, PPIN, TPIN, ENPIN1, ENPIN2>,
    command_queue: Arc<freertos_rust::Queue<Command>>,
    ic: Arc<dyn IInterruptController>,
    mut processor: TP,
) -> !
where
    PTIM: InCounter<PDMA, PPIN>,
    TTIM: InCounter<TDMA, TPIN>,
    ENPIN1: OutputPin,
    <ENPIN1 as OutputPin>::Error: Debug,
    ENPIN2: OutputPin,
    <ENPIN2 as OutputPin>::Error: Debug,
    TP: RawValueProcessor,
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
            FChannel::Pressure,
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
            FChannel::Temperature,
        ),
    );

    //----------------------------------------------------
    let mut initial_target = |ch| {
        processor
            .process_f_signal_lost(ch, crate::config::INITIAL_FREQMETER_TARGET)
            .1
            .unwrap()
            .1
    };

    let cc = command_queue.clone();
    let mut p_controller = FreqmeterController::new(
        &mut perith.timer1,
        perith.en_1,
        FChannel::Pressure,
        initial_target(FChannel::Pressure),
        move || {
            cc.send(
                Command::TimeoutFChannel(FChannel::Pressure),
                Duration::infinite(),
            )
            .unwrap();
        },
    );

    let cc = command_queue.clone();
    let mut t_controller = FreqmeterController::new(
        &mut perith.timer2,
        perith.en_2,
        FChannel::Temperature,
        initial_target(FChannel::Temperature),
        move || {
            cc.send(
                Command::TimeoutFChannel(FChannel::Temperature),
                Duration::infinite(),
            )
            .unwrap();
        },
    );

    let mut p_channels: [&mut dyn FChProcessor; 2] = [&mut p_controller, &mut t_controller];

    //----------------------------------------------------

    loop {
        if let Ok(cmd) = command_queue.receive(Duration::infinite()) {
            match cmd {
                Command::Start(Channel::FChannel(c)) => p_channels[c as usize].enable(),
                Command::Stop(Channel::FChannel(c)) => p_channels[c as usize].diasble(),
                Command::ReadyFChannel(c, ev, target, captured, wraped) => {
                    if wraped {
                        defmt::warn!("Ch: {}, wrap detected", c);
                    }

                    let ch = &mut p_channels[c as usize];

                    if let Some(result) = ch.input_captured(ev, captured) {
                        let (continue_work, new_target) =
                            processor.process_f_result(c, target, result);

                        if let Some((nt, mt)) = new_target {
                            ch.set_target(nt, mt);
                        }

                        if continue_work {
                            if cfg!(feature = "freqmeter-start-stop") {
                                ch.restart();
                            } else {
                                ch.reset_guard();
                            }
                        } else {
                            ch.diasble();
                        }
                    } else {
                        #[cfg(not(feature = "freqmeter-start-stop"))]
                        defmt::trace!("Ch. {}, result not ready", c)
                    }
                }
                Command::SetTarget(c, target, mt) => p_channels[c as usize].set_target(target, mt),
                Command::TimeoutFChannel(c) => {
                    defmt::warn!("ch. {} signal lost.", c);
                    let ch = &mut p_channels[c as usize];

                    ch.diasble();

                    let (continue_work, new_target) =
                        processor.process_f_signal_lost(c, ch.target());
                    if continue_work {
                        if let Some((nt, mt)) = new_target {
                            ch.set_target(nt, mt);
                        }
                        ch.enable();
                    }
                }
                _ => {}
            }
        }
    }
}
