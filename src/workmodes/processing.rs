use crate::threads::sensor_processor::{AChannel, FChannel};

pub trait RawValueProcessor {
    /// Процессинг:
    /// а. кладет результат в выходные значения
    /// б. проверяет, на стоит ли выключить счетчик прямо сейчас
    /// в. вычисляет частоту, если нужно
    /// г. вычисляет выходные занчения, если нужно
    /// д. если нужно, вычисляет новое значение для адаптации
    /// е. возвращет: разрешение продолжения работы, новое значение для адаптации
    fn process_f_result(&mut self, ch: FChannel, target: u32, result: u32) -> (bool, Option<u32>);

    fn process_adc_result(&mut self, ch: AChannel, result: u32) -> bool;
}

mod common;
pub use common::{
    abs_difference, calc_freq, calc_new_target, calc_pressure, calc_temperature, channel_config,
};

mod high_performance;
pub use high_performance::HighPerformanceProcessor;
