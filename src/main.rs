#![no_std]
#![no_main]

use core::convert::TryInto;

use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys::esp_ds_sign;

use log::info;

const SSID: &str = "2030/32Greenwich";
const PASSWORD: &str = "0xPARC123";

#[no_mangle]
fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop,
        timer_service,
    )
    .unwrap();

    block_on(connect_wifi(&mut wifi));

    info!("wifi connected, getting ip");

    let ip_info = wifi.wifi().sta_netif().get_ip_info().unwrap();

    info!("Wifi DHCP info: {:?}", ip_info);
}

async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>) {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration).unwrap();

    wifi.start().await.unwrap();
    info!("Wifi started");

    wifi.connect().await.unwrap();
    info!("Wifi connected");

    wifi.wait_netif_up().await.unwrap();
    info!("Wifi netif up");
}
