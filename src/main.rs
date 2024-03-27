use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::Ipv4Addr;
use chrono::Local;
use std::path::Path;
use std::io;
use std::io::BufRead;
use std::str::FromStr;
// use serde::{Deserialize};
use serde_json::Value;

// the below strict can be used to get the current ip address
// #[derive(Deserialize, Debug)]
// struct go_daddy_get_record {
//     data: String
//     , name: String
//     , port: Option<u32>
//     , priority: Option<u32>
//     , protocol: Option<String>
//     , service: Option<String>
//     , ttl: u32
//     , r#type: String
//     , weight: Option<u32>
// }

fn get_ip() -> Result<Ipv4Addr, Box<dyn std::error::Error>> {
    let ip_string = reqwest::blocking::get("https://api.ipify.org?format=json")?
        .json::<HashMap<String, String>>()?
        .get("ip")
        .cloned();
    let ip = match ip_string {
        Some(ip_string_value) => Ipv4Addr::from_str(ip_string_value.as_str())?
        , _ => {
            log_error_to_file_and_panic(format!("Error when converting ip string to IpV4Addr").as_str());
            panic!() // Line will never be reached as the above function ends in a panic
        }
    };
    Ok(ip)
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn log_error_to_file_and_panic(error_message: &str) -> () {
    let error_file_name = "error.txt";
    let error_file_open_error_message = format!("Could not open file {}", error_file_name);
    let mut error_file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(error_file_name)
        .expect(&error_file_open_error_message);
    let current_local_time = Local::now().to_rfc2822();
    let error_string = format!("{}: {}\n", current_local_time, error_message);
    error_file.write_all(&error_string.into_bytes()).expect(format!("Could not write error to file {}", error_file_name).as_str());
    panic!("{}", error_message);
}

fn record_ip_and_send(new_ip: Ipv4Addr, ip_history_file_name: &str) -> () {

    // credentials were stripped out here to make public

    let mut ip_history_file = match OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(ip_history_file_name) {
        Ok(file) => file,
        _ => {
            log_error_to_file_and_panic(format!("Could not create the file {}", ip_history_file_name).as_str());
            panic!() // Line will never be reached as the above function ends in a panic
        }
    };
    ip_history_file.write_all(&format!("{}: {}\n", Local::now().to_rfc2822(), &new_ip).into_bytes())
        .expect(format!("Could not write to file {}", ip_history_file_name).as_str());

    // let records_url = "https://api.ote-godaddy.com/v1/domains/ccapsuna.com/records/A/@";
    let records_url = "https://api.godaddy.com/v1/domains/ccapsuna.com/records/A/@";
    let mut ip_update_body: Vec<HashMap<&str, Value>> = Vec::new();
    let mut new_record: HashMap<&str, Value> = HashMap::new();
    new_record.insert("ttl", serde_json::json!(600));
    new_record.insert("data", serde_json::json!(new_ip.to_string()));
    ip_update_body.push(new_record);
    let serialized_ip_update_body = serde_json::to_string(&ip_update_body).unwrap();

    // println!("{serialized_ip_update_body}");

    // // How to get the current ip from GoDaddy

    // // let request = client.get(records_url)
    // //     // .key_secret_auth(ote_key, Some(ote_secret));
    // //     .key_secret_auth(prod_key, Some(prod_secret))
    // // let response = request.send()
    // //     .expect("can't go wrong");
    // //     // .text();
    // //     .json::<Vec<go_daddy_get_record>>()
    // //     .expect("can't go wrong");

    // ////////////////
    // // How to update the ip on GoDaddy

    let client = reqwest::blocking::Client::new();
    let request = client.put(records_url)
        // .key_secret_auth(ote_key, Some(ote_secret));
        .key_secret_auth(prod_key, Some(prod_secret))
        // without the below line the api rejects the request because it thinks
        // the data is of type octetstream
        .header("Content-Type", "application/json")
        .body(serialized_ip_update_body);
    
    ////////////////
        // The below code can help to inspect the request if there are troubles

    // let built_request = request.build().unwrap();
    // let request_body = built_request.body();
    // println!("{request_body:#?}");

        ///////////////
        // The below code can show the body from a response

    // let response = request.send()
    //     .expect("can't go wrong");
    //     // .text();

        ///////////////

    let response = request.send()
        .expect("can't go wrong");
    // println!("{response:#?}");
    // let response_body = response.text();
    // println!("{response_body:#?}");
    let response_status = response.status();
    println!("{response_status:#?}")
    /////////////////
}

fn main() {

    let current_ip_result = get_ip();
    // let current_ip_option = match current_ip_result {
    let current_ip = match current_ip_result {
        Ok(ip_option) => ip_option
        , _ => {
            log_error_to_file_and_panic("IP retrieval produced an error");
            panic!() // Line will never be reached as the above function ends in a panic
        }
    };

    println!("{:#?}", current_ip);

    let ip_history_file_name = "ip_history.txt";

    match read_lines(ip_history_file_name) {
        Ok(lines) => {
            let mut last_line_content = "".to_string();
            for line in lines {
                last_line_content = match line {
                    Ok(content) => content,
                    _ => {
                        log_error_to_file_and_panic(format!("Error when reading line from file {}", &ip_history_file_name).as_str());
                        panic!() // Line will never be reached as the above function ends in a panic
                    }
                };
            };
            let ip_string = match last_line_content.split(": ").last() {
                Some(item) => item
                , _ => {
                    log_error_to_file_and_panic(format!("Could not split the last line from {} and grab the last element", &ip_history_file_name).as_str());
                    panic!() // Line will never be reached as the above function ends in a panic
                }
            };
            let last_recorded_ip_option = Ipv4Addr::from_str(ip_string);
            let last_recorded_ip = match last_recorded_ip_option {
                Ok(ip) => ip
                , _ => {
                    log_error_to_file_and_panic("Last recorded ip produced an error");
                    panic!() // Line will never be reached as the above function ends in a panic
                }
            };
            if last_recorded_ip != current_ip {
                record_ip_and_send(current_ip, ip_history_file_name);
            }
        }
        , _ => {
            record_ip_and_send(current_ip, ip_history_file_name);
        }
    };
}