extern "C" {
    pub fn pb_decode(
        stream: *mut pb_istream_t,
        fields: *const pb_msgdesc_t,
        dest_struct: *mut ::core::ffi::c_void,
    ) -> bool;

    pub fn pb_decode_ex(
        stream: *mut pb_istream_t,
        fields: *const pb_msgdesc_t,
        dest_struct: *mut ::core::ffi::c_void,
        flags: u32,
    ) -> bool;

    #[doc = " Functions for manipulating streams *"]
    pub fn pb_istream_from_buffer(buf: *const pb_byte_t, msglen: size_t) -> pb_istream_t;

    pub fn pb_read(stream: *mut pb_istream_t, buf: *mut pb_byte_t, count: size_t) -> bool;

    #[doc = " Helper functions for writing field callbacks *"]
    pub fn pb_decode_tag(
        stream: *mut pb_istream_t,
        wire_type: *mut pb_wire_type_t,
        tag: *mut u32,
        eof: *mut bool,
    ) -> bool;

    pub fn pb_skip_field(stream: *mut pb_istream_t, wire_type: pb_wire_type_t) -> bool;

    pub fn pb_decode_varint(stream: *mut pb_istream_t, dest: *mut u64) -> bool;

    pub fn pb_decode_varint32(stream: *mut pb_istream_t, dest: *mut u32) -> bool;

    pub fn pb_decode_bool(stream: *mut pb_istream_t, dest: *mut bool) -> bool;

    pub fn pb_decode_svarint(stream: *mut pb_istream_t, dest: *mut i64) -> bool;

    pub fn pb_decode_fixed32(stream: *mut pb_istream_t, dest: *mut ::core::ffi::c_void) -> bool;

    pub fn pb_decode_fixed64(stream: *mut pb_istream_t, dest: *mut ::core::ffi::c_void) -> bool;

    pub fn pb_make_string_substream(
        stream: *mut pb_istream_t,
        substream: *mut pb_istream_t,
    ) -> bool;

    pub fn pb_close_string_substream(
        stream: *mut pb_istream_t,
        substream: *mut pb_istream_t,
    ) -> bool;
}