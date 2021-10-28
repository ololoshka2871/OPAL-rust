extern "C" {
    pub fn pb_encode(
        stream: *mut pb_ostream_t,
        fields: *const pb_msgdesc_t,
        src_struct: *const ::core::ffi::c_void,
    ) -> bool;

    pub fn pb_encode_ex(
        stream: *mut pb_ostream_t,
        fields: *const pb_msgdesc_t,
        src_struct: *const ::core::ffi::c_void,
        flags: u32,
    ) -> bool;

    pub fn pb_get_encoded_size(
        size: *mut size_t,
        fields: *const pb_msgdesc_t,
        src_struct: *const ::core::ffi::c_void,
    ) -> bool;

    #[doc = " Functions for manipulating streams *"]
    pub fn pb_ostream_from_buffer(buf: *mut pb_byte_t, bufsize: size_t) -> pb_ostream_t;

    pub fn pb_write(stream: *mut pb_ostream_t, buf: *const pb_byte_t, count: size_t) -> bool;

    #[doc = " Helper functions for writing field callbacks *"]
    pub fn pb_encode_tag_for_field(
        stream: *mut pb_ostream_t,
        field: *const pb_field_iter_t,
    ) -> bool;

    pub fn pb_encode_tag(
        stream: *mut pb_ostream_t,
        wiretype: pb_wire_type_t,
        field_number: u32,
    ) -> bool;

    pub fn pb_encode_varint(stream: *mut pb_ostream_t, value: u64) -> bool;

    pub fn pb_encode_svarint(stream: *mut pb_ostream_t, value: i64) -> bool;

    pub fn pb_encode_string(
        stream: *mut pb_ostream_t,
        buffer: *const pb_byte_t,
        size: size_t,
    ) -> bool;

    pub fn pb_encode_fixed32(stream: *mut pb_ostream_t, value: *const ::core::ffi::c_void) -> bool;

    pub fn pb_encode_fixed64(stream: *mut pb_ostream_t, value: *const ::core::ffi::c_void) -> bool;

    pub fn pb_encode_submessage(
        stream: *mut pb_ostream_t,
        fields: *const pb_msgdesc_t,
        src_struct: *const ::core::ffi::c_void,
    ) -> bool;
}
