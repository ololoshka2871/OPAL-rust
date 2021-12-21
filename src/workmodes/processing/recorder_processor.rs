use core::ops::DerefMut;

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
                Ok(FChCfg {
                    p_preheat_time_ms: sysclk
                        .duration_ms(ws.PMesureTime_ms * crate::config::PREHEAT_MULTIPLIER)
                        .to_ms(),
                    t_preheat_time_ms: sysclk
                        .duration_ms(ws.TMesureTime_ms * crate::config::PREHEAT_MULTIPLIER)
                        .to_ms(),
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
            })
            .unwrap(),

            adaptate_f: Arc::new(Mutex::new(false).unwrap()),
        }
    }

    pub fn start<W, D, P>(&mut self, writer: W, led: P) -> Result<Task, FreeRtosError>
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
        let start_delay = sysclk.duration_ms(super::recorder_start_delay().unwrap());

        let cfg = self.ch_cfg.clone();
        let fm = self.fref_multiplier;

        let adaptate_f = self.adaptate_f.clone();

        Task::new()
            .name("RecCtrl")
            .stack_size(512)
            .priority(TaskPriority(config::RECORDER_CTRL_PRIO))
            .start(move |_| {
                Self::led_blink(led, config::START_BLINK_COUNT, blink_period);
                CurrentTask::delay(start_delay); // задержка старта
                Self::controller(output, cq, cfg, fm, writer, adaptate_f);
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
    ) where
        W: WriteController<D>,
        D: DataPage,
    {
        let adaptate_req = move |req| {
            adaptate_f
                .lock(Duration::infinite())
                .map(|mut g| *g = req)
                .unwrap();
        };

        let enable_p_channel = |enabled| {
            if enabled {
                commad_queue
                    .send(
                        Command::Start(Channel::FChannel(FChannel::Pressure)),
                        Duration::infinite(),
                    )
                    .unwrap();
            }
        };

        let enable_t_channel = |enabled| {
            if enabled {
                commad_queue
                    .send(
                        Command::Start(Channel::FChannel(FChannel::Temperature)),
                        Duration::infinite(),
                    )
                    .unwrap();
            }
        };

        let start_analog_channels = |tcpu_en, vbat_en| {
            if tcpu_en {
                commad_queue
                    .send(
                        Command::Start(Channel::AChannel(AChannel::TCPU)),
                        Duration::infinite(),
                    )
                    .unwrap();
            }
            if vbat_en {
                commad_queue
                    .send(
                        Command::Start(Channel::AChannel(AChannel::Vbat)),
                        Duration::infinite(),
                    )
                    .unwrap();
            }
        };

        fn calc_fp(o: &mut OutputStorage, fm: f64) {
            for c in [FChannel::Pressure, FChannel::Temperature] {
                if let Some(result) = o.results[c as usize] {
                    if let Ok(f) = super::calc_freq(fm, o.targets[c as usize], result) {
                        o.frequencys[c as usize] = Some(f);
                        match c {
                            FChannel::Pressure => super::calc_pressure(f, o),
                            FChannel::Temperature => super::calc_temperature(f, o),
                        }
                    }
                } else {
                    o.frequencys[c as usize] = None;
                    o.values[c as usize] = None;
                }
            }
        }

        let push_data = |page: &mut D, ch: FChannel| -> bool {
            output
                .lock(Duration::infinite())
                .map(|guard| page.push_data(guard.results[ch as usize], ch))
                .unwrap()
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
            CurrentTask::delay(Duration::ms(ch_cfg.p_preheat_time_ms));
        }

        // 2. Частотыне каналы прогреты, включаем аналоговые
        start_analog_channels(ch_cfg.tcpu_en, ch_cfg.vbat_en);
        CurrentTask::delay(Duration::ticks(10));

        loop {
            // 3. Создаем новый буфер страницы флеш-памяти
            let mut page = loop {
                match writer.new_page() {
                    Ok(p) => break p,
                    Err(e) => {
                        defmt::error!("{}, retrying...", defmt::Debug2Format(&e));
                        CurrentTask::delay(Duration::ticks(10));
                    }
                }
            };
            output
                .lock(Duration::infinite())
                .map(|mut guard| {
                    let o = guard.deref_mut();
                    calc_fp(o, fm);
                    page.write_header(o);
                })
                .unwrap();

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

            // 7. Запись станицы
            match writer.start_write(page) {
                PageWriteResult::Succes(n) => {
                    defmt::info!("Flash page {} writen", n);
                }
                PageWriteResult::Fail(e) => {
                    defmt::error!("Flash page write error: {}", e);
                }
                PageWriteResult::MemoryFull => {
                    defmt::warn!("Last page writen, power down!");
                    super::halt_cpu();
                }
            }

            // 8. Ожидание завершения адаптации
            CurrentTask::delay(Duration::ms(core::cmp::max(
                ch_cfg.p_preheat_time_ms,
                ch_cfg.t_preheat_time_ms,
            )));

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
        if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
            guard.targets[ch as usize] = target;
            guard.results[ch as usize] = Some(result);
        }

        // продолжить работу, только если интервал записи меньше или равен времени прогрева канала
        let continue_work = match ch {
            FChannel::Pressure => self.ch_cfg.p_preheat_time_ms < self.ch_cfg.p_write_period_ms,
            FChannel::Temperature => self.ch_cfg.t_preheat_time_ms < self.ch_cfg.t_write_period_ms,
        };

        if self
            .adaptate_f
            .lock(Duration::infinite())
            .map(|g| *g)
            .unwrap()
        {
            let mut new_cfg = None;
            if let Ok(f) = super::calc_freq(self.fref_multiplier, target, result) {
                if let Ok((new_target, new_guard)) = super::calc_new_target(ch, f, &self.sysclk) {
                    new_cfg = Some((new_target, new_guard));
                    defmt::warn!(
                        "Ch. {} ({} Hz) Adaptation requested, target {} -> {}",
                        ch,
                        f,
                        target,
                        new_target
                    );
                }
            }

            (true, new_cfg)
        } else {
            (continue_work, None)
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

        super::guard_ticks(ch, &self.sysclk)
            .map(|g| (true, Some((target, g))))
            .unwrap()
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