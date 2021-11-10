extern crate alloc;
use alloc::boxed::Box;

use crate::common::{
    pb_callback_s__bindgen_ty_1, pb_callback_t, pb_field_iter_t, pb_msgdesc_t, pb_ostream_t,
};

pub fn new_tx_callback(
    f: Box<dyn Fn(&mut pb_ostream_t, &pb_field_iter_t) -> bool>,
) -> pb_callback_t {
    unsafe extern "C" fn wraper(
        out_stream: *mut pb_ostream_t,
        field: *const pb_field_iter_t,
        arg: *const *mut ::core::ffi::c_void,
    ) -> bool {
        let f = (*arg) as *const Box<dyn Fn(&mut pb_ostream_t, &pb_field_iter_t) -> bool>;
        (*f)(&mut *out_stream, &*field)
    }

    let arg = Box::new(f);

    pb_callback_t {
        funcs: pb_callback_s__bindgen_ty_1 {
            encode: Some(wraper),
        },
        arg: Box::into_raw(arg) as *mut _,
    }
}

pub trait TxRepeated {
    fn reset(&mut self);
    fn has_next(&mut self) -> bool;
    fn encode_next(&self, out_stream: &mut pb_ostream_t) -> Result<(), crate::Error>;
    fn fields(&self) -> &'static pb_msgdesc_t;
}

pub fn new_tx_repeated_callback(f: Box<dyn TxRepeated>) -> pb_callback_t {
    unsafe extern "C" fn wraper(
        out_stream: *mut pb_ostream_t,
        field: *const pb_field_iter_t,
        arg: *const *mut ::core::ffi::c_void,
    ) -> bool {
        let f = (*arg) as *mut Box<dyn TxRepeated>;

        (*f).reset();
        loop {
            if (*f).has_next() {
                if (*out_stream).encode_tag_for_field(&*field).is_err() {
                    return false;
                }

                if (*f).encode_next(&mut *out_stream).is_err() {
                    return false;
                }
            } else {
                return true;
            }
        }
    }

    let arg = Box::new(f);

    pb_callback_t {
        funcs: pb_callback_s__bindgen_ty_1 {
            encode: Some(wraper),
        },
        arg: Box::into_raw(arg) as *mut _,
    }
}

impl Drop for pb_callback_t {
    fn drop(&mut self) {
        if !self.arg.is_null() {
            let b = unsafe { Box::from_raw(self.arg as *mut Box<dyn Fn()>) };
            drop(b);
        }
    }
}
