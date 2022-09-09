use embedded_hal::digital::v2::OutputPin;

pub trait ParallelOutputBus<T: OutputPin> {
    type Output;
    fn set(&self, value: Self::Output);
}

#[macro_export]
macro_rules! simple_parallel_output_bus {
    ($name:ident: $valtype:ty => ($(pin $pint:ty),+)) => {
        struct $name($( pub $pint ),* );

        impl<T: embedded_hal::digital::v2::OutputPin>
            crate::support::parallel_output_bus::ParallelOutputBus<T> for $name {
            type Output = $valtype;

            fn set(&self, value: Self::Output) {
                $(
                    {
                        ${ignore(pint)}
                        let i = ${index()};
                        if value & (1 << i) != 0 {
                            self.${index()}.set_high();
                        } else {
                            self.${index()}.set_low();
                        }
                    }
                )* 
            }
        }
    }
}
