#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(deprecated)]

use crate::data_provider;

include!("bindings/bindings-decoder.rs");

impl Default for _heatshrink_decoder {
    fn default() -> _heatshrink_decoder {
        unsafe { core::mem::uninitialized() }
    }
}

pub struct HeatshrinkDecoder<'a, T>
where
    T: Iterator<Item = u8>,
{
    ctx: _heatshrink_decoder,
    finished: bool,

    // Поскольку это трейт а не объект нужно чтобы ссылка жила не меньше чем сама структура
    src: &'a mut T,
}

impl<'a, T> HeatshrinkDecoder<'a, T>
where
    T: Iterator<Item = u8>,
{
    pub fn from_source(src: &'a mut T) -> HeatshrinkDecoder<T> {
        let mut res = HeatshrinkDecoder {
            ctx: _heatshrink_decoder::default(),
            finished: false,
            src, // то же что src: src
        };
        unsafe {
            heatshrink_decoder_reset(&mut res.ctx);
        }
        res
    }
}

impl<'a, T> Iterator for HeatshrinkDecoder<'a, T>
where
    T: Iterator<Item = u8>,
{
    type Item = u8; // byte

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut outbuf: u8 = 0;
            let mut actualy_read: usize = 0;
            let res = unsafe {
                heatshrink_decoder_poll(&mut self.ctx, &mut outbuf, 1, &mut actualy_read)
            };
            match res {
                HSDR_sink_res_HSER_POLL_EMPTY => {
                    if actualy_read == 0 {
                        if self.finished {
                            return None;
                        }
                    } else {
                        return Some(outbuf);
                    }
                }
                HSDR_sink_res_HSER_POLL_MORE => {
                    // ok
                    if actualy_read == 1 {
                        return Some(outbuf);
                    } else {
                        panic!(
                            "heatshrink_encoder_poll: Requested read 1 byte, but {} got",
                            actualy_read
                        );
                    }
                }
                HSDR_sink_res_HSER_POLL_ERROR_NULL => panic!("Nullptr!"), /* NULL argument */
                HSDR_sink_res_HSER_POLL_ERROR_MISUSE => panic!(),         /* API misuse */
            }

            // need more data
            let d = self.src.next();
            if !d.is_none() {
                let mut actualy_read: usize = 0;
                let mut in_buf = d.unwrap();
                let mut res = unsafe {
                    heatshrink_decoder_sink(&mut self.ctx, &mut in_buf, 1, &mut actualy_read)
                };
                match res {
                    HSD_sink_res_HSDR_SINK_OK => {}                // ok
                    HSD_sink_res_HSDR_SINK_ERROR_NULL => panic!(), // buffer full
                    HSD_sink_res_HSER_SINK_ERROR_NULL => panic!("Nullptr!"),
                    N => panic!("Unknown result heatshrink_decoder_sink: {}", N),
                }
            } else {
                // try finalise
                self.finished = true;
                let res = unsafe { heatshrink_decoder_finish(&mut self.ctx) };
                match res {
                    HSDR_finish_res_HSER_FINISH_DONE => return None, // ok
                    HSDR_finish_res_HSER_FINISH_ERROR_NULL => panic!("Nullptr!"),
                    HSDR_finish_res_HSER_FINISH_MORE => {} // there is data in encoder buff
                    N => panic!("Unknown result heatshrink_decoder_finish: {}", N),
                }
            }
        }
    }
}
