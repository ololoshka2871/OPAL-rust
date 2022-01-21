use core::{cmp::max, ops::DerefMut};

use alloc::sync::Arc;
use freertos_rust::{CurrentTask, Duration, FreeRtosError, Mutex, Queue, Task, TaskPriority};
use stm32l4xx_hal::{adc::ADC, prelude::OutputPin, time::Hertz};

use crate::{
    main_data_storage::{
        data_page::DataPage,
        write_controller::{PageWriteResult, WriteController},
    },
    sensors::analog::AController,
    threads::sensor_processor::{AChannel, Channel, Command, FChannel},
    workmodes::{common::HertzExt, output_storage::OutputStorage},
};

use super::RawValueProcessor;

#[derive(Copy, Clone)]
pub struct FChCfg {
    pub p_preheat_time_ms: u32,
    pub t_preheat_time_ms: u32,
    pub p_write_period_ms: u32,
    pub t_write_period_ms: u32,

    pub p_en: bool,
    pub t_en: bool,
    pub tcpu_en: bool,
    pub vbat_en: bool,
}

pub struct RecorderProcessor {
    output: Arc<Mutex<OutputStorage>>,
    commad_queue: Arc<Queue<Command>>,
    fref_multiplier: f64,
    sysclk: Hertz,

    ch_cfg: FChCfg,
    adaptate_f: Arc<Mutex<bool>>,
}

impl RecorderProcessor {
    pub fn new(
        output: Arc<Mutex<OutputStorage>>,
        cq: Arc<Queue<Command>>,
        fref_multiplier: f64,
        sysclk: Hertz,
    ) -> Self {
        Self {
            output,
            commad_queue: cq,
            fref_multiplier,
            sysclk,

            ch_cfg: super::read_settings(|(ws, _)| {
                let preheat_time_ms = |mt| {
                    sysclk
                        .duration_ms(max(
                            mt * crate::config::PREHEAT_MULTIPLIER
                                + crate::config::GEN_COLD_STARTUP_TIME_MS,
                            crate::config::MINIMUM_ADAPTATION_INTERVAL
                                + crate::config::GEN_COLD_STARTUP_TIME_MS,
                        ))
                        .to_ms()
                };

                Ok(FChCfg {
                    p_preheat_time_ms: preheat_time_ms(ws.PMesureTime_ms),
                    t_preheat_time_ms: preheat_time_ms(ws.TMesureTime_ms),
                    p_write_period_ms: sysclk
                        .duration_ms(ws.writeConfig.BaseInterval_ms * ws.writeConfig.PWriteDevider)
                        .to_ms(),
                    t_write_period_ms: sysclk
                        .duration_ms(ws.writeConfig.BaseInterval_ms * ws.writeConfig.TWriteDevider)
                        .to_ms(),
                    p_en: ws.P_enabled,
                    t_en: ws.T_enabled,
                    tcpu_en: ws.TCPUEnabled,
                    vbat_en: ws.VBatEnabled,
                })
            }),
            adaptate_f: Arc::new(Mutex::new(false).unwrap()),
        }
    }

    pub fn start<W, D, P>(
        &mut self,
        scb: cortex_m::peripheral::SCB,
        writer: W,
        led: P,
    ) -> Result<Task, FreeRtosError>
    where
        P: 'static + OutputPin + Send,
        W: 'static + WriteController<D>,
        D: 'static + DataPage,
    {
        use crate::config;

        let output = self.output.clone();
        let cq = self.commad_queue.clone();
        let sysclk = self.sysclk;

        let blink_period = sysclk.duration_ms(config::START_BLINK_PERIOD_MS);
        let start_delay = sysclk.duration_ms(super::recorder_start_delay());

        let cfg = self.ch_cfg.clone();
        let fm = self.fref_multiplier;

        let adaptate_f = self.adaptate_f.clone();

        Task::new()
            .name("RecCtrl")
            .stack_size(1024)
            .priority(TaskPriority(config::RECORDER_CTRL_PRIO))
            .start(move |_| {
                Self::led_blink(led, config::START_BLINK_COUNT, blink_period);
                CurrentTask::delay(start_delay); // задержка старта
                Self::controller(output, cq, cfg, fm, writer, adaptate_f, scb);
            })
    }

    fn led_blink<P>(mut _led: P, _cout: u32, _period: Duration)
    where
        P: OutputPin,
    {
        #[cfg(feature = "led-blink")]
        for _ in 0..cout * 2 {
            use crate::config;

            let _ = led.set_state(config::LED_ENABLE);
            CurrentTask::delay(Duration::ms(period.to_ms() / 2));
            let _ = led.set_state(config::LED_DISABLE);
            CurrentTask::delay(Duration::ms(period.to_ms() / 2));
        }
    }

    fn controller<W, D>(
        output: Arc<Mutex<OutputStorage>>,
        commad_queue: Arc<Queue<Command>>,
        ch_cfg: FChCfg,
        fm: f64,
        mut writer: W,
        adaptate_f: Arc<Mutex<bool>>,
        mut scb: cortex_m::peripheral::SCB,
    ) where
        W: WriteController<D>,
        D: DataPage,
    {
        let adaptate_req = move |req| {
            let _ = adaptate_f.lock(Duration::infinite()).map(|mut g| *g = req);
        };

        let send_cc = |cmd| {
            let _ = commad_queue.send(cmd, Duration::infinite());
        };

        let enable_p_channel = |enabled| {
            if enabled {
                send_cc(Command::Start(Channel::FChannel(FChannel::Pressure)));
            }
        };

        let enable_t_channel = |enabled| {
            if enabled {
                send_cc(Command::Start(Channel::FChannel(FChannel::Temperature)));
            }
        };

        let start_analog_channels = |tcpu_en, vbat_en| {
            if tcpu_en {
                send_cc(Command::Start(Channel::AChannel(AChannel::TCPU)));
            }
            if vbat_en {
                send_cc(Command::Start(Channel::AChannel(AChannel::Vbat)));
            }
        };

        fn calc_fp(o: &mut OutputStorage, fm: f64) {
            for c in [FChannel::Pressure, FChannel::Temperature] {
                if let Some(result) = o.results[c as usize] {
                    let f = super::calc_freq(fm, o.targets[c as usize], result);

                    o.frequencys[c as usize] = Some(f);
                    match c {
                        FChannel::Pressure => super::calc_pressure(f, o),
                        FChannel::Temperature => super::calc_temperature(f, o),
                    }
                } else {
                    o.frequencys[c as usize] = None;
                    o.values[c as usize] = None;
                }
            }
        }

        let push_data = |page: &mut D, ch: FChannel| -> bool {
            unsafe {
                output
                    .lock(Duration::infinite())
                    .map(|guard| page.push_data(guard.results[ch as usize], ch))
                    .unwrap_unchecked()
            }
        };

        //1. Включаем частотыне каналы
        adaptate_req(true);
        if ch_cfg.p_preheat_time_ms != ch_cfg.t_preheat_time_ms {
            if ch_cfg.p_preheat_time_ms < ch_cfg.t_preheat_time_ms {
                enable_t_channel(ch_cfg.t_en);
                CurrentTask::delay(Duration::ms(
                    ch_cfg.t_preheat_time_ms - ch_cfg.p_preheat_time_ms,
                ));
                enable_p_channel(ch_cfg.p_en);
                CurrentTask::delay(Duration::ms(ch_cfg.p_preheat_time_ms));
            } else {
                enable_p_channel(ch_cfg.p_en);
                CurrentTask::delay(Duration::ms(
                    ch_cfg.p_preheat_time_ms - ch_cfg.t_preheat_time_ms,
                ));
                enable_t_channel(ch_cfg.t_en);
                CurrentTask::delay(Duration::ms(ch_cfg.t_preheat_time_ms));
            }
        } else {
            enable_p_channel(ch_cfg.p_en);
            enable_t_channel(ch_cfg.t_en);
            // 2 - по тому, что каналы включаются синхронно (там delay)
            CurrentTask::delay(Duration::ms(2 * ch_cfg.p_preheat_time_ms));
        }
        adaptate_req(false);

        // 2. Частотыне каналы прогреты, включаем аналоговые
        start_analog_channels(ch_cfg.tcpu_en, ch_cfg.vbat_en);
        CurrentTask::delay(Duration::ticks(10));

        loop {
            // 3. Создаем новый буфер страницы флеш-памяти
            let mut page = loop {
                match writer.try_create_new_page() {
                    Ok(p) => break p,
                    Err(freertos_rust::FreeRtosError::OutOfMemory) => {
                        super::halt_cpu(
                            &mut scb,
                            "Memory full, power down after 1s",
                            Duration::ms(1_000),
                        );
                    }
                    Err(e) => {
                        defmt::error!("{}, retrying...", defmt::Debug2Format(&e));
                        CurrentTask::delay(Duration::ticks(10));
                    }
                }
            };
            let v_bat = unsafe {
                output
                    .lock(Duration::infinite())
                    .map(|mut guard| {
                        let o = guard.deref_mut();
                        calc_fp(o, fm);
                        page.write_header(o);
                        o.vbat
                    })
                    .unwrap_unchecked()
            };

            let _ = crate::settings::settings_action::<_, _, _, ()>(
                Duration::infinite(),
                |(settings, _)| {
                    if v_bat < settings.VbatWorkRange.minimum {
                        super::halt_cpu(
                            &mut scb,
                            "Battery low, power down after 1s",
                            Duration::ms(1_000),
                        );
                    }
                    Ok(())
                },
            );

            // 4. Сбор данных
            let mut to_p_write = ch_cfg.p_write_period_ms;
            let mut to_t_write = ch_cfg.t_write_period_ms;
            loop {
                let to_next_write = core::cmp::min(to_p_write, to_t_write);
                CurrentTask::delay(Duration::ms(to_next_write));
                to_p_write -= to_next_write;
                to_t_write -= to_next_write;

                if to_p_write == 0 {
                    if push_data(&mut page, FChannel::Pressure) {
                        break;
                    }
                    to_p_write = ch_cfg.p_write_period_ms;
                }

                if to_t_write == 0 {
                    if push_data(&mut page, FChannel::Temperature) {
                        break;
                    }
                    to_t_write = ch_cfg.t_write_period_ms;
                }
            }

            // 5. Финализация
            page.finalise();

            // 6. Запрос адаптации
            adaptate_req(true);

            let write_time = {
                let start_moment = freertos_rust::FreeRtosUtils::get_tick_count();

                let write_res = writer.write(page);

                let end_moment = freertos_rust::FreeRtosUtils::get_tick_count();

                let mut write_time = if end_moment >= start_moment {
                    end_moment - start_moment
                } else {
                    freertos_rust::FreeRtosTickType::MAX - start_moment + end_moment
                };

                // 7. Запись станицы
                match write_res {
                    PageWriteResult::Succes(n) => {
                        defmt::info!("Flash page {} writen ({} ms)", n, write_time);
                    }
                    PageWriteResult::Fail(e) => {
                        defmt::error!("Flash page write error: {}", e);
                        write_time = 0;
                    }
                }

                write_time
            };

            // 8. Ожидание завершения адаптации пропуская 1 измерение
            CurrentTask::delay(Duration::ms(
                (core::cmp::max(ch_cfg.p_preheat_time_ms, ch_cfg.t_preheat_time_ms)
                    / crate::config::PREHEAT_MULTIPLIER
                    + crate::config::MINIMUM_ADAPTATION_INTERVAL)
                    .checked_sub(write_time)
                    .unwrap_or_default(),
            ));

            adaptate_req(false);
        }
    }
}

impl RawValueProcessor for RecorderProcessor {
    // Выходное значение не считается, только сырые значения записываются на выход
    // Канал должен сам уснуть если
    // 1. это 2 цыкла
    // 2. период записи больше чем время прогрева канала
    fn process_f_result(
        &mut self,
        ch: FChannel,
        target: u32,
        result: u32,
    ) -> (bool, Option<(u32, u32)>) {
        /*
        defmt::debug!(
            "process_f_result(ch={}, target={}, result{})",
            ch,
            target,
            result
        );
        */

        if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
            guard.targets[ch as usize] = target;
            guard.results[ch as usize] = Some(result);
        }

        if unsafe {
            self.adaptate_f
                .lock(Duration::infinite())
                .map(|g| *g)
                .unwrap_unchecked()
        } {
            let f = super::calc_freq(self.fref_multiplier, target, result);
            let (new_target, new_guard) = super::calc_new_target(ch, f, &self.sysclk);
            let new_cfg = Some((new_target, new_guard));

            defmt::warn!(
                "Ch. {} ({} Hz) Adaptation requested, target {} -> {}",
                ch,
                f,
                target,
                new_target
            );

            (true, new_cfg)
        } else {
            (
                // продолжить работу, только если интервал записи меньше или равен времени "прогрева" канала
                match ch {
                    FChannel::Pressure => {
                        self.ch_cfg.p_preheat_time_ms > self.ch_cfg.p_write_period_ms
                    }
                    FChannel::Temperature => {
                        self.ch_cfg.t_preheat_time_ms > self.ch_cfg.t_write_period_ms
                    }
                },
                None,
            )
        }
    }

    fn process_f_signal_lost(&mut self, ch: FChannel, target: u32) -> (bool, Option<(u32, u32)>) {
        // отвал сигнала на входе, сброс значений
        if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
            guard.targets[ch as usize] = target;
            guard.results[ch as usize] = None;
            guard.frequencys[ch as usize] = None;
            guard.values[ch as usize] = None;
        }

        let guard_ticks = super::guard_ticks(ch, &self.sysclk);
        (true, Some((target, guard_ticks)))
    }

    fn process_adc_result(
        &mut self,
        ch: AChannel,
        current_period_ticks: u32,
        adc: &mut ADC,
        controller: &mut dyn AController,
    ) -> (bool, Option<u32>) {
        let raw_adc_value = controller.read(adc);

        match ch {
            AChannel::TCPU => super::process_t_cpu(
                self.output.as_ref(),
                current_period_ticks,
                adc.to_degrees_centigrade(raw_adc_value),
                raw_adc_value,
                self.sysclk,
            ),
            AChannel::Vbat => super::process_vbat(
                self.output.as_ref(),
                current_period_ticks,
                adc.to_millivolts(raw_adc_value),
                raw_adc_value,
                self.sysclk,
            ),
        };

        // запрет цыклического выполнения 1 измерение в шапку и все.
        (false, None)
    }
}
