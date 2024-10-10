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
use esp_idf_sys::sr::{aec_destroy, aec_pro_create, aec_process};

use log::info;

const SSID: &str = "2030/32Greenwich";
const PASSWORD: &str = "0xPARC123";

#[no_mangle]
fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    unsafe {
        // Create AEC handle
        let aec_handle = aec_pro_create(16, 1, 0);

        const BUFFER_SIZE: usize = 256;

        // Create input buffers
        let mut far_end = [0i16; BUFFER_SIZE];
        let mut near_end = [0i16; BUFFER_SIZE];
        let mut out = [0i16; BUFFER_SIZE];

        // Simulate some audio data
        for i in 0..BUFFER_SIZE {
            far_end[i] = ((i as i16 * 7) % 1000) - 500; // Sawtooth-like pattern
            near_end[i] = ((i as i16 * 11) % 1200) - 600; // Different sawtooth-like pattern
        }

        // Process AEC
        aec_process(
            aec_handle,
            far_end.as_mut_ptr(),
            near_end.as_mut_ptr(),
            out.as_mut_ptr(),
        );

        // Log a subset of the results (first 10 samples)
        info!(
            "AEC Input (Far-end) [first 10 samples]: {:?}",
            &far_end[..10]
        );
        info!(
            "AEC Input (Near-end) [first 10 samples]: {:?}",
            &near_end[..10]
        );
        info!("AEC Output [first 10 samples]: {:?}", &out[..10]);

        aec_destroy(aec_handle);
    }

    // let mut afe_config = afe_config_t::default();
    // unsafe {
    //     let a = esp_afe_sr_v1;
    //     if let Some(create_func) = a.create_from_config {
    //         let afe_handle = create_func(&mut afe_config as *mut afe_config_t);
    //         // Use afe_handle as needed
    //         info!("afe handle {:?}", afe_handle);
    //     } else {
    //         // Handle the case where create_from_config is None
    //         info!("create_from_config function is not available");
    //     }
    // }

    // let peripherals = Peripherals::take().unwrap();
    // let sys_loop = EspSystemEventLoop::take().unwrap();
    // let timer_service = EspTaskTimerService::new().unwrap();
    // let nvs = EspDefaultNvsPartition::take().unwrap();

    // let mut wifi = AsyncWifi::wrap(
    //     EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
    //     sys_loop,
    //     timer_service,
    // )
    // .unwrap();

    // block_on(connect_wifi(&mut wifi));

    // info!("wifi connected, getting ip");

    // let ip_info = wifi.wifi().sta_netif().get_ip_info().unwrap();

    // info!("Wifi DHCP info: {:?}", ip_info);
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
