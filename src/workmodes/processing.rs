use crate::{
    sensors::analog::AController,
    threads::sensor_processor::{AChannel, FChannel},
};

pub trait RawValueProcessor {
    /// Процессинг:
    /// а. кладет результат в выходные значения
    /// б. проверяет, на стоит ли выключить счетчик прямо сейчас
    /// в. вычисляет частоту, если нужно
    /// г. вычисляет выходные занчения, если нужно
    /// д. если нужно, вычисляет новое значение для адаптации
    /// е. возвращет: разрешение продолжения работы, новое значение для адаптации
    fn process_f_result(
        &mut self,
        ch: FChannel,
        target: u32,
        result: u32,
    ) -> (bool, Option<(u32, u32)>);

    fn process_f_signal_lost(&mut self, ch: FChannel, target: u32) -> (bool, Option<(u32, u32)>);

    fn process_adc_result(
        &mut self,
        ch: AChannel,
        current_period_ticks: u32,
        adc: &mut ADC,
        controller: &mut dyn AController,
    ) -> (bool, Option<u32>);
}

mod common;
pub use common::{
    abs_difference, calc_freq, calc_new_target, calc_pressure, calc_temperature, channel_config,
    guard_ticks, process_t_cpu, process_vbat,
};

mod high_performance;
pub use high_performance::HighPerformanceProcessor;
use stm32l4xx_hal::adc::ADC;
