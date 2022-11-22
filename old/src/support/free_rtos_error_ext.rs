use defmt::{write, Format, Formatter};
use freertos_rust::FreeRtosError;

pub struct FreeRtosErrorContainer(pub FreeRtosError);

impl Format for FreeRtosErrorContainer {
    fn format(&self, fmt: Formatter) {
        static VARIANTS_STR: [&str; 10] = [
            "OutOfMemory",
            "QueueSendTimeout",
            "QueueReceiveTimeout",
            "MutexTimeout",
            "Timeout",
            "QueueFull",
            "StringConversionError",
            "TaskNotFound",
            "InvalidQueueSize",
            "ProcessorHasShutDown",
        ];

        write!(fmt, "{}", VARIANTS_STR[self.0 as usize]);
    }
}
