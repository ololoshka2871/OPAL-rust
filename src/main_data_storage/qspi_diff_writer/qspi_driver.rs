#[cfg(any(feature = "stm32l433", feature = "stm32l443"))]
use qspi_stm32lx3::qspi as qspi;

#[cfg(not(any(feature = "stm32l433", feature = "stm32l443")))]
use stm32l4xx_hal::qspi as qspi;


use qspi::{Qspi, QspiConfig, QspiMode, QspiReadCommand};
