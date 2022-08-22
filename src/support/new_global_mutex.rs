use freertos_rust::Mutex;

#[allow(unused)]
pub fn new_global_mutex<T>() -> Mutex<Option<T>> {
    Mutex::new(None).unwrap()
}
