pub trait ParallelOutputBus {
    type Output;
    fn set(&mut self, value: Self::Output);
}

// pin_id

#[macro_export]
macro_rules! simple_parallel_output_bus {
    ($name:ident: $valtype:ty => ($(pin $pint:ty),+)) => {
        pub struct $name($( pub $pint ),* );

        impl crate::support::parallel_output_bus::ParallelOutputBus for $name {
            type Output = $valtype;

            fn set(&mut self, value: Self::Output) {
                $(
                    {
                        ${ignore($pint)}
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
