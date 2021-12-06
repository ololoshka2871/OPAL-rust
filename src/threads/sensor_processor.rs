use alloc::sync::Arc;
use freertos_rust::{Duration, InterruptContext};

use crate::{
    sensors::{
        freqmeter::{
            master_counter::{MasterCounter, MasterTimerInfo},
            InCounter, OnCycleFinished,
        },
        Enable,
    },
    support::interrupt_controller::{IInterruptController, Interrupt},
};

pub struct SensorPerith<TIM1, DMA1, TIM2, DMA2, PIN1, PIN2>
// Суть в том, что мы напишем КОНКРЕТНУЮ имплементацию InCounter<DMA> для
// конкретного счетчика рандомная пара не соберется.
where
    TIM1: InCounter<DMA1, PIN1>,
    TIM2: InCounter<DMA2, PIN2>,
{
    pub timer1: TIM1,
    pub timer1_dma_ch: DMA1,
    pub timer1_pin: PIN1,
    pub timer2: TIM2,
    pub timer2_dma_ch: DMA2,
    pub timer2_pin: PIN2,
}

#[derive(Clone, Copy, Debug)]
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

struct DMAFinished {
    master: MasterTimerInfo,
    cc: Arc<freertos_rust::Queue<Command>>,
    ic: Arc<dyn IInterruptController>,
    ch: Channel,
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

pub fn sensor_processor<PTIM, PDMA, TTIM, TDMA, PPIN, TPIN>(
    mut perith: SensorPerith<PTIM, PDMA, TTIM, TDMA, PPIN, TPIN>,
    command_queue: Arc<freertos_rust::Queue<Command>>,
    ic: Arc<dyn IInterruptController>,
) -> !
where
    PTIM: InCounter<PDMA, PPIN> + Enable,
    TTIM: InCounter<TDMA, TPIN> + Enable,
{
    let master_counter = MasterCounter::allocate().unwrap();
    perith.timer1.configure(
        master_counter.cnt_addr(),
        &mut perith.timer1_dma_ch,
        perith.timer1_pin,
        ic.as_ref(),
        DMAFinished {
            master: master_counter,
            cc: command_queue.clone(),
            ic: ic.clone(),
            ch: Channel::Pressure,
        },
    );

    let master_counter = MasterCounter::allocate().unwrap();
    perith.timer2.configure(
        master_counter.cnt_addr(),
        &mut perith.timer2_dma_ch,
        perith.timer2_pin,
        ic.as_ref(),
        DMAFinished {
            master: master_counter,
            cc: command_queue.clone(),
            ic: ic.clone(),
            ch: Channel::Temperature,
        },
    );

    let mut channels: [&mut dyn Enable; 2] = [&mut perith.timer1, &mut perith.timer2];

    //------------------ remove after tests --------------
    crate::settings::settings_action::<_, _, _, ()>(Duration::infinite(), |(ws, _)| {
        let flags = [ws.P_enabled, ws.T_enabled, ws.TCPUEnabled, ws.VBatEnabled];
        for (c, f) in channels.iter_mut().zip(flags.iter()) {
            if *f {
                (*c).start();
            }
        }

        Ok(())
    })
    .expect("Failed to read channel enable");
    //-----------------------------------------------------

    loop {
        if let Ok(cmd) = command_queue.receive(Duration::zero()) {
            //defmt::warn!("sensors command: {}", defmt::Debug2Format(&cmd));
            match cmd {
                Command::Start(c) => {
                    channels[c as usize].start();
                }
                Command::Stop(c) => {
                    channels[c as usize].stop();
                }
                Command::Ready(_c, _result, _target, _wraped) => {}
            }
        }
    }
}
