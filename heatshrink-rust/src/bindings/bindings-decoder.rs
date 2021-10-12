
pub const HEATSHRINK_AUTHOR: &'static [u8; 32usize] = b"Scott Vokes <vokes.s@gmail.com>\0";
pub const HEATSHRINK_URL: &'static [u8; 43usize] = b"https://github.com/atomicobject/heatshrink\0";

pub type size_t = usize;

pub const HSD_sink_res_HSDR_SINK_OK: HSD_sink_res = 0;
pub const HSD_sink_res_HSDR_SINK_FULL: HSD_sink_res = 1;
pub const HSD_sink_res_HSDR_SINK_ERROR_NULL: HSD_sink_res = -1;
pub type HSD_sink_res = i32;
pub const HSD_poll_res_HSDR_POLL_EMPTY: HSD_poll_res = 0;
pub const HSD_poll_res_HSDR_POLL_MORE: HSD_poll_res = 1;
pub const HSD_poll_res_HSDR_POLL_ERROR_NULL: HSD_poll_res = -1;
pub const HSD_poll_res_HSDR_POLL_ERROR_UNKNOWN: HSD_poll_res = -2;
pub type HSD_poll_res = i32;
pub const HSD_finish_res_HSDR_FINISH_DONE: HSD_finish_res = 0;
pub const HSD_finish_res_HSDR_FINISH_MORE: HSD_finish_res = 1;
pub const HSD_finish_res_HSDR_FINISH_ERROR_NULL: HSD_finish_res = -1;
pub type HSD_finish_res = i32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _heatshrink_decoder {
    pub input_size: u16,
    pub input_index: u16,
    pub output_count: u16,
    pub output_index: u16,
    pub head_index: u16,
    pub state: u8,
    pub current_byte: u8,
    pub bit_index: u8,
    pub buffers: [u8; 288usize],
}

extern "C" {
    pub fn heatshrink_decoder_reset(hsd: *mut _heatshrink_decoder);
    pub fn heatshrink_decoder_sink(
        hsd: *mut _heatshrink_decoder,
        in_buf: *mut u8,
        size: size_t,
        input_size: *mut size_t,
    ) -> HSD_sink_res;
    pub fn heatshrink_decoder_poll(
        hsd: *mut _heatshrink_decoder,
        out_buf: *mut u8,
        out_buf_size: size_t,
        output_size: *mut size_t,
    ) -> HSD_poll_res;
    pub fn heatshrink_decoder_finish(hsd: *mut _heatshrink_decoder) -> HSD_finish_res;
}
