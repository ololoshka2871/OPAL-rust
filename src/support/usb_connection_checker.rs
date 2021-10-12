pub trait UsbConnectionChecker {
    fn is_usb_connected(&self) -> bool;
}
