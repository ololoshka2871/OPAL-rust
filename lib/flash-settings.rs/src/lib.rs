#![no_std]

use core::marker::PhantomData;

pub enum LoadError<T> {
    ReadError(T),
    ConststenceError,
}

pub trait StoragePolicy<T> {
    unsafe fn store(&mut self, data: &[u8]) -> Result<(), T>;
    unsafe fn load(&mut self, data: &mut [u8]) -> Result<(), LoadError<T>>;
}

pub struct SettingsManager<T: 'static, U, Terr, Tpolicy: StoragePolicy<Terr>> {
    polcy: Tpolicy,
    work_copy: T,
    non_store_values: U,
    default: &'static T,
    _phantomdata2: PhantomData<Terr>,
}

impl<T, U, Terr, Tpolicy> SettingsManager<T, U, Terr, Tpolicy>
where
    T: Copy + Sized,
    Tpolicy: StoragePolicy<Terr>,
{
    pub fn load(&mut self) -> Result<(), LoadError<Terr>> {
        unsafe {
            self.polcy.load(core::slice::from_raw_parts_mut(
                &mut self.work_copy as *mut T as *mut _,
                core::mem::size_of::<T>(),
            ))
        }
    }

    pub fn save(&mut self) -> Result<(), Terr> {
        unsafe {
            self.polcy.store(core::slice::from_raw_parts(
                (&self.work_copy as *const T) as *const u8,
                core::mem::size_of::<T>(),
            ))
        }
    }

    pub fn ref_mut(&mut self) -> (&mut T, &mut U) {
        (&mut self.work_copy, &mut self.non_store_values)
    }

    pub fn new(default: &'static T, polcy: Tpolicy, non_store_values_init: U) -> Self {
        let mut res = Self {
            polcy,
            work_copy: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            non_store_values: non_store_values_init,
            default,
            _phantomdata2: PhantomData,
        };

        if let Err(e) = res.load() {
            match e {
                LoadError::<Terr>::ConststenceError => {
                    res.work_copy = *res.default;
                    if res.save().is_err() {
                        panic!("Failed to save defailt settings")
                    }
                }
                LoadError::<Terr>::ReadError(_) => panic!("Failed to init settings"),
            }
        }

        res
    }

    pub fn polcy(&mut self) -> &mut Tpolicy {
        &mut self.polcy
    }
}
