#[allow(unused)]
pub fn len_in_u64_aligned(data: &[u8]) -> usize {
    if data.len() % ::core::mem::size_of::<u64>() != 0 {
        data.len() / ::core::mem::size_of::<u64>() + 1
    } else {
        data.len() / ::core::mem::size_of::<u64>()
    }
}
