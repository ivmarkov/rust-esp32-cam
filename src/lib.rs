#![allow(clippy::single_component_path_imports)]

use std::{sync::Arc, thread, time::*};

use anyhow::*;
use log::*;

use embedded_svc::ping::Ping;
use embedded_svc::wifi::*;

use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;

const SSID: &str = "ssid";
const PASS: &str = "pass";

#[no_mangle]
fn main() -> ! {
    // Get backtraces from anyhow; only works for Xtensa arch currently
    #[cfg(arch = "xtensa")]
    env::set_var("RUST_BACKTRACE", "1");

    let _wifi = wifi().unwrap();

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn wifi() -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(
        Arc::new(EspNetifStack::new()?),
        Arc::new(EspSysLoopStack::new()?),
        Arc::new(EspDefaultNvs::new()?),
    )?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected, about to do some pings");

        //info!("{:#}", std::backtrace::Backtrace::capture());

        let ping_summary =
            ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
        if ping_summary.transmitted != ping_summary.received {
            bail!(
                "Pinging gateway {} resulted in timeouts",
                ip_settings.subnet.gateway
            );
        }

        info!("Pinging done");
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}
