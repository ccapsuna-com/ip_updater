#![allow(non_camel_case_types)] 
use std::{
    fs::File
    , fs::remove_file
    , net::Ipv4Addr
    , path::Path
    , io
    , io::BufRead
    , str::FromStr
    , env
    , time::Duration
    , time::Instant
    , thread::sleep
    , process::Command
};
use serde::{Serialize
    ,Deserialize
};
// use serde_json::Value;
use reqwest::header::{HeaderMap, HeaderValue};
use log::{info, error, LevelFilter};
use log4rs::append::{console::ConsoleAppender, file::FileAppender};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
use http::header;

#[derive(Deserialize, Debug)]
struct auth_info {
    key: String
    , zone: String
    , record: String
}

#[derive(Serialize, Debug)]
struct cloudflare_patch_record_request<'a> {
    name: &'a str
    , r#type: &'a str
    , content: Ipv4Addr
}

// #[derive(Deserialize, Debug)]
// struct cloudflare_get_records_response {
//     result: Vec<couldflare_result>
//     , success: bool
//     , errors: Vec<String>
//     , messages: Vec<String>
//     , result_info: cloudflare_result_info
// }
// #[derive(Deserialize, Debug)]
// struct cloudflare_result_info {
//     page: u32
//     , per_page: u32
//     , count: u32
//     , total_count: u32
//     , total_pages: u32
// }
// #[derive(Deserialize, Debug)]
// struct couldflare_result {
//     id: String
//     , zone_id: String
//     , zone_name: String
//     , name: String
//     , r#type: String
//     , content: String
//     , proxiable: bool
//     , proxied: bool
//     , ttl: u32
//     , locked: bool
//     , meta: cloudflare_meta
//     , comment: Option<String>
//     , tags: Vec<String>
//     , created_on: String
//     , modified_on: String
// }
// #[derive(Deserialize, Debug)]
// struct cloudflare_meta {
//     auto_added: bool
//     , managed_by_apps: bool
//     , managed_by_argo_tunnel: bool
// }

#[derive(Deserialize, Debug)]
struct new_ip_response {
    ip: String
}

fn log_error_and_panic(error_message: String) {
    error!("{error_message}");
    release_lock();
    panic!("{error_message}")
}

fn get_ip() -> Ipv4Addr {
    let ip_string_response = reqwest::blocking::get("https://api.ipify.org?format=json")
        .unwrap_or_else(|e|{
            log_error_and_panic(format!("Could not get new ip response. Error message was:\n\n{e}"));
            unreachable!()
        });
    let ip_string_response_text = ip_string_response.text()
        .unwrap_or_else(|e|{
            log_error_and_panic(format!("Could not convert ip response to text. Error was:\n\n{e}"));
            unreachable!()
        });
    let ip_string_struct: new_ip_response = serde_json::from_str(ip_string_response_text.as_str())
        .unwrap_or_else(|e|{
            log_error_and_panic(format!(
                "Could not convert ip response to new_ip_response. Response was:\n\n{}\n\nError was:\n\n{}"
                , ip_string_response_text
                , e
            ));
            unreachable!()
        });
    Ipv4Addr::from_str(ip_string_struct.ip.as_str()).unwrap_or_else(|e|{
            log_error_and_panic(format!(
                "Could not convert ip response to new_ip_response. IP from response text was:\n\n{}\n\nError was:\n\n{}"
                , ip_string_response_text
                , e
            ));
            unreachable!()
        })
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_auth_info() -> auth_info {
    let mut auth_data_command = Command::new("gpg");
    auth_data_command.args(["--decrypt", "/home/ip_updater/.ip_updater_auth.gpg"]);
    let auth_data_command_output = auth_data_command.output().unwrap_or_else(|e|{
            log_error_and_panic(format!(
                "Decryption did not run successfully. Error was:\n\n{}"
                , e
            ));
            unreachable!()
        });
    let output_message = String::from_utf8(auth_data_command_output.stdout).expect("Terminal output is always valid utf8");
    let output_error = String::from_utf8(auth_data_command_output.stderr).expect("Terminal output is always valid utf8");
    let output_status_code = auth_data_command_output.status.code().unwrap_or_else(||{
            log_error_and_panic(format!(
                "Decryption status code was None"
            ));
            unreachable!()
        });
    if output_status_code == 0 {
        serde_json::from_str::<auth_info>(output_message.as_str()).unwrap_or_else(|e|{
            log_error_and_panic(format!(
                "Could not convert decryption output to auth_info. Decryption output was:\n\n{}\n\nError was:\n\n{}"
                , output_message
                , e
            ));
            unreachable!()
        })
    } else {
        log_error_and_panic(format!(
            "Decryption status code was not 0. Decryption output was:\n\n{}\n\nDecryption error was:\n\n{}"
            , output_message
            , output_error
        ));
        unreachable!()
    }
}

fn record_ip_and_send(new_ip: Ipv4Addr) -> () {
    let auth_stuff = get_auth_info();
    let api_key = format!("Bearer {}", auth_stuff.key);
    let api_url = "https://api.cloudflare.com/client/v4";
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    let mut custom_header_value = HeaderValue::from_str(api_key.as_str()).unwrap_or_else(|e|{
        log_error_and_panic(format!(
            "Could not create header value. Error was:\n\n{}"
            , e
        ));
        unreachable!()
    });
    custom_header_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, custom_header_value);
    
    // How to get the current ip from Cloudflare
    
    // let call_url = format!("{}/zones/{}/dns_records", api_url, auth_stuff.zone);
    // let request = client.get(call_url)
    //     .headers(headers);
    // let response: cloudflare_get_records_response = request.send()
    //     .expect("can't go wrong")
    //     // .text();
    //     .json()
    //     .expect("can't go wrong");
    // println!("{response:#?}");
    ////////////////

    // The below code can help to inspect the request if there are troubles

    // let built_request = request.build().unwrap();
    // // let request_body = built_request.body();
    // let request_header = built_request.headers();
    // // println!("{request_body:#?}");
    // println!("{request_header:#?}");
    ///////////////

    // The below code can show the body from a response

    // let response = request.send()
    //     .expect("can't go wrong");
    //     // .text();

    // println!("{response:#?}");
    // let response_body = response.text();
    // println!("{response_body:#?}");
    /////////////////

    let ip_update_body = cloudflare_patch_record_request {
        name: "ccapsuna.com"
        , r#type: "A"
        , content: new_ip
    };
    let serialized_ip_update_body = serde_json::to_string(&ip_update_body).unwrap_or_else(|e|{
        log_error_and_panic(format!(
            "Could not serialize path record struct. Struct was:\n\n{ip_update_body:#?}\n\nError was:\n\n{}"
            , e
        ));
        unreachable!()
    });

    // println!("{serialized_ip_update_body}");

    // // How to update the ip on Cloudflare

    let call_url = format!("{}/zones/{}/dns_records/{}", api_url, auth_stuff.zone, auth_stuff.record);
    let request = client.patch(call_url)
        .headers(headers)
        .body(serialized_ip_update_body);

    let response = request.send().unwrap_or_else(|e|{
        log_error_and_panic(format!(
            "Error when trying to send request. Error was:\n\n{}"
            , e
        ));
        unreachable!()
    });
    if response.status().is_success() {
        info!(target: "history_logger", "The new ip is: {}", &new_ip);
        info!("IP updated successfully");
    } else {
        let response_text = response.text().unwrap_or_else(|e|{
            log_error_and_panic(format!(
                "IP update request could not be converted to text. Error was:\n\n{}"
                , e
            ));
            unreachable!()
        });
        log_error_and_panic(format!(
            "IP update response did not return 200. Response was:\n\n{}"
            , response_text
        ));
        unreachable!()
    };
}

fn release_lock() -> () {
    let mut lock_released = false;
    let loop_start_time = Instant::now();
    while lock_released == false && loop_start_time.elapsed() < Duration::from_secs(5) {
        let lock_file_removal = remove_file(LOCK_FILE_NAME);
        lock_released = match lock_file_removal {
            Ok(_) => true
            , Err(_) => {
                sleep(Duration::from_secs_f32(0.5));
                false
            }
        }
    }
    if lock_released == false {
        error!("Lock file could not be released within 5 seconds alloted");
        panic!()
    }
}

const LOGS_ROOT_LOCATION: &str = "/var/log/cc_app_logs/ip_updater/";
const IP_HISTORY_FILE_NAME: &str = "ip_history.log";
const MAIN_LOG_FILE_NAME: &str = "main.log";
const LOCK_FILE_NAME: &str = "/opt/bin/ip_updater.lock";
const FILE_LOG_OUTPUT_FORMAT: &str = "{d(%Y-%m-%d %H:%M:%S %Z)(utc)} {l} {t} - {m}{n}";

fn main() {
    ////// Parameters

    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "3".to_string());

    ////// Creating lock file
    let loop_start_time = Instant::now();
    let mut lock_acquired = false;
    while lock_acquired == false && loop_start_time.elapsed() < Duration::from_secs(5) {
        let lock_file = File::create_new(LOCK_FILE_NAME);
        lock_acquired = match lock_file {
            Ok(_) => true
            , Err(_) => {
                sleep(Duration::from_secs_f32(0.5));
                false
            }
        }
    }

    ////// Logger

    let level_filter = match log_level.as_str() {
        "0" => LevelFilter::Off,
        "1" => LevelFilter::Error,
        "2" => LevelFilter::Warn,
        "4" => LevelFilter::Debug,
        "5" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    // Creating the console logger
    let stdout = ConsoleAppender::builder().build();
    // Creating the file logger for for the project with a specific output format and location
    let log_file_appender_result = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(FILE_LOG_OUTPUT_FORMAT)))
        .build(format!("{LOGS_ROOT_LOCATION}{MAIN_LOG_FILE_NAME}"));
    let log_file_appender = log_file_appender_result.unwrap_or_else(|e|{
        release_lock();
        panic!("Could not create main log file appender. Error was:\n\n{e}");
    });
    // Configuring the ip history logger
    let history_file_appender_result = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(FILE_LOG_OUTPUT_FORMAT)))
        .build(format!("{LOGS_ROOT_LOCATION}{IP_HISTORY_FILE_NAME}"));
    let history_file_appender = history_file_appender_result.unwrap_or_else(|e|{
        release_lock();
        panic!("Could not create ip history file appender. Error was:\n\n{e}");
    });
    // Configuring the logger
    let logger_config_result = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("main_log", Box::new(log_file_appender)))
        .appender(Appender::builder().build("history_log", Box::new(history_file_appender)))
        .logger(Logger::builder().appender("history_log").build("history_logger", level_filter))
        .build(Root::builder()
            .appender("stdout")
            .appender("main_log")
            .build(level_filter)
        );
    let logger_config = logger_config_result.unwrap_or_else(|e|{
        release_lock();
        panic!("Error when building the logger config. Error was:\n\n{e}");
    });

    log4rs::init_config(logger_config).unwrap_or_else(|e|{
        release_lock();
        panic!("Error when initializing the logger. Error was:\n\n{e}");
    });

    ////// Main logic
    if lock_acquired == false {
        log_error_and_panic(format!(
                "Lock file could not be create since it is already present and did not \
                disappear within 5 seconds of the program start"
            ));
            unreachable!();
    }
    let current_ip = get_ip();
    match read_lines(format!("{LOGS_ROOT_LOCATION}{IP_HISTORY_FILE_NAME}")) {
        Ok(lines) => {
            let last_line = lines.last().unwrap_or(Ok("".to_string())).unwrap_or("".to_string());
            let ip_string = last_line.split(" ").last().unwrap_or("");
            match Ipv4Addr::from_str(ip_string) {
                Ok(last_ip) => {
                    if last_ip != current_ip {
                        record_ip_and_send(current_ip);
                        release_lock();
                    } else {
                        info!("IP has not changed.");
                        release_lock();
                    }
                }
                , Err(_) => {
                    record_ip_and_send(current_ip);
                    release_lock()
                }
            }
        }
        , Err(_) => {
            record_ip_and_send(current_ip);
            release_lock()
        }
    };
}
