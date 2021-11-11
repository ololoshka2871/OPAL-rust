#![no_std]

use core::marker::PhantomData;

pub trait StoragePolicy<T> {
    unsafe fn store(&self, data: &[u8]) -> Result<(), T>;
    unsafe fn load(&self, data: &mut [u8]) -> Result<(), T>;
}

pub struct SettingsManager<T, Terr, Tpolicy: StoragePolicy<Terr>> {
    polcy: Tpolicy,
    _phantomdata1: PhantomData<T>,
    _phantomdata2: PhantomData<Terr>,
}

impl<T, Terr, Tpolicy> SettingsManager<T, Terr, Tpolicy>
where
    T: Copy + Sized,
    Tpolicy: StoragePolicy<Terr>,
{
    pub fn store(&self, obj: &T) -> Result<(), Terr> {
        unsafe {
            self.polcy.store(core::slice::from_raw_parts(
                (obj as *const T) as *const u8,
                core::mem::size_of::<T>(),
            ))
        }
    }

    pub fn load(&self) -> Result<T, Terr> {
        let mut buf: T = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        if let Err(e) = unsafe {
            self.polcy.load(core::slice::from_raw_parts_mut(
                &mut buf as *mut T as *mut _,
                core::mem::size_of::<T>(),
            ))
        } {
            Err(e)
        } else {
            Ok(buf)
        }
    }

    pub fn new(polcy: Tpolicy) -> Self {
        Self {
            polcy: polcy,
            _phantomdata1: PhantomData,
            _phantomdata2: PhantomData,
        }
    }
}
