pub trait ParallelInputBus {
    type Input;
    fn get(&self) -> Self::Input;
}

#[macro_export]
macro_rules! simple_parallel_input_bus {
    ($name:ident: $valtype:ty => ($(pin $pint:ty),+)) => {
        struct $name($( pub $pint ),* );

        impl crate::support::parallel_input_bus::ParallelInputBus for $name {
            type Input = $valtype;

            fn get(&self) -> $valtype {
                let mut res = 0;
                $(
                    {
                        ${ignore(pint)}
                        let i = ${index()};
                        if self.${index()}.is_high() {
                            res |= 1 << i;
                        }
                    }
                )* 
                res
            }
        }
    }
}
