use freertos_rust::FreeRtosError;

pub enum PageWriteResult {
    Succes(u32),
    Fail(u32),
}

pub trait WriteController<P>: Send {
    fn try_create_new_page(&mut self) -> Result<P, FreeRtosError>;
    fn write(&mut self, page: P) -> PageWriteResult;
}
