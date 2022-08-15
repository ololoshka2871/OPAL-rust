#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use serde::Serialize;

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct AppSettings {
    pub Delay: u32,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct NonStoreSettings;
