use core::{ptr::addr_of_mut, sync::atomic::AtomicBool};

//use stm32f1xx_hal::pac::interrupt;

// 1. Таймер триггерит DMA которая копирует u16 из памяти в GPIO (32)
// 2. В буфере 40 u16 -> это 20 CLOCKов
// 3. Прерывание DMA считает переданное и как только накопится 20 останавливает процесс.
// 4. Если второй буфер готов к передаче буферы свапются и сразу начинается отправка
// 5. Загрузка новой команды всегда в теневой буфер.

const TX_POCKET_SIZE: usize = 20;

static mut OUTPUT_BUF_A: [u16; TX_POCKET_SIZE * 2] = [0; TX_POCKET_SIZE * 2];
static mut OUTPUT_BUF_B: [u16; TX_POCKET_SIZE * 2] = [0; TX_POCKET_SIZE * 2];

static mut TX_BUF: *mut [u16] = unsafe { addr_of_mut!(OUTPUT_BUF_A) };
static mut BACK_BUF: *mut [u16] = unsafe { addr_of_mut!(OUTPUT_BUF_B) };

static mut BACK_BUF_READY: AtomicBool = AtomicBool::new(false);

pub trait XY2_100Interface {
    fn begin(&mut self, tim_ref_clk: stm32f1xx_hal::time::Hertz);
    fn set_pos(&mut self, x: u16, y: u16);
}

pub struct XY2_100<TIMER, DMACH, OUTPUTS> {
    timer: TIMER,
    dma: DMACH,
    port_addr: u32,
    outputs: OUTPUTS,
}

//static mut DMA1_CH2_IT: Option<unsafe fn()> = None;

pub mod tim2_gpiob_3456;

/*
#[interrupt]
unsafe fn DMA1_CHANNEL2() {
    (DMA1_CH2_IT.expect("DMA1_CHANNEL2 not registred!"))();
}
*/

fn build_msg(data: u16) -> u32 {
    // ... [0 0 1 <data16> <parity>] = 20 bit total
    let mut res = (0b001u32 << 17) | ((data as u32) << 1);
    res |= parity(res);
    res
}

pub fn parity(v: u32) -> u32 {
    v.count_ones() % 2
}
