/// can call from any place
/// --Rust--
/// extern "C" {
///     pub fn log_i32_anywhere(v: i32);
/// }
///
/// --or (C/C++)--
///
/// extern void log_i32_anywhere(int32_t v);
/// ...
/// unsafe { log_i32_anywhere(v); }
#[no_mangle]
pub extern "C" fn log_i32_anywhere(v: i32) {
    defmt::debug!("log_i32_anywhere({})", v);
}
