#![no_std]
#![no_main]

use rp_pico as bsp;

use core::{fmt::Write, str::from_utf8};

use bsp::{
    entry,
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        gpio::{FunctionSpi, FunctionUart},
        pac::{CorePeripherals, Peripherals},
        sio::Sio,
        uart::{DataBits, StopBits, UartConfig, UartPeripheral},
        watchdog::Watchdog,
        Spi,
    },
    XOSC_CRYSTAL_FREQ,
};
use cortex_m::delay::Delay;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_hal::spi::MODE_0;
use fugit::RateExtU32;
use panic_probe as _;
use picolony::{
    device::{ssd1351::create_ssd1351_graphics, uart::UartPeripheralIoExt},
    graphics::{
        drawables::{JisText, JisTextStyle},
        font::{JisFont, JisFont8x12, JisFontInterface},
    },
    string::FormatBuffer,
};

const FONT_K8X12_BITMAP: &[u8] = include_bytes!("../assets/k8x12.bin");
const FONT_CACHE_SIZE: usize = 256;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);
    let core = CorePeripherals::take().unwrap();
    let sio = Sio::new(peripherals.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = bsp::Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Set up pins
    let uart_tx = pins.gpio0.into_mode::<FunctionUart>();
    let uart_rx = pins.gpio1.into_mode::<FunctionUart>();
    let _spi_sclk = pins.gpio2.into_mode::<FunctionSpi>();
    let _spi_mosi = pins.gpio3.into_mode::<FunctionSpi>();
    let _spi_miso = pins.gpio4.into_mode::<FunctionSpi>();
    let _spi_cs = pins.gpio5.into_mode::<FunctionSpi>();
    let oled_dc = pins.gpio6.into_push_pull_output();
    let mut oled_rst = pins.gpio7.into_push_pull_output();

    let mut uart = UartPeripheral::new(
        peripherals.UART0,
        (uart_tx, uart_rx),
        &mut peripherals.RESETS,
    )
    .enable(
        UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
        clocks.peripheral_clock.freq(),
    )
    .unwrap();

    let spi = Spi::<_, _, 8>::new(peripherals.SPI0);
    let spi = spi.init(
        &mut peripherals.RESETS,
        clocks.peripheral_clock.freq(),
        8_000_000u32.Hz(),
        &MODE_0,
    );
    let mut oled_display = create_ssd1351_graphics(spi, oled_dc);

    // Draw display
    oled_display.reset(&mut oled_rst, &mut delay).unwrap();
    oled_display.init().unwrap();

    // Initialize uni2jis table and font
    let mut format_buffer = FormatBuffer::<1024>::new();
    let font_k8x12 = JisFont::<JisFont8x12, FONT_CACHE_SIZE>::new(&FONT_K8X12_BITMAP).unwrap();
    let white_k8x12 = JisTextStyle::new(&font_k8x12, Rgb565::WHITE);

    let mut buffer = [0; 1024];
    loop {
        let Ok(size) = uart.read_line(&mut buffer, false) else { continue };
        let Ok(read_str) = from_utf8(&buffer[..size]) else { continue };
        format_buffer.clear();
        format_buffer.write_str(read_str).unwrap();

        oled_display.clear();
        JisText::new(format_buffer.valid_str(), Point::new(0, 0), &white_k8x12)
            .draw(&mut oled_display)
            .unwrap();
    }
}
