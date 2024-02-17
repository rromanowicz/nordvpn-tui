use std::process::{Command, Output};

#[derive(Debug)]
pub struct Nord {
    pub status: Status,
    pub countries: Vec<Country>,
}

impl Nord {
    pub fn get_country(&self, name: String) -> &Country {
        return self
            .countries
            .iter()
            .find(|it| it.name == name)
            .expect("Country not found");
    }

    pub fn refresh_status(&mut self) {
        self.status = get_status();
    }
}

#[derive(Debug)]
pub struct Status {
    pub status: String,
    pub ip: String,
    pub country: String,
    pub city: String,
    pub transfer: Transfer,
    pub uptime: String,
}

#[derive(Debug)]
pub struct Transfer {
    pub down: String,
    pub up: String,
}

#[derive(Debug)]
pub struct Country {
    pub name: String,
    pub cities: Vec<City>,
}

#[derive(Clone, Debug)]
pub struct City {
    pub name: String,
}

pub trait NordList {
    fn name(&self) -> String;
}

impl NordList for Country {
    fn name(&self) -> String {
        String::from(&self.name)
    }
}

impl NordList for City {
    fn name(&self) -> String {
        String::from(&self.name)
    }
}

pub fn init() -> Nord {
    Nord {
        status: get_status(),
        countries: get_countries(),
    }
}

pub fn get_countries() -> Vec<Country> {
    let output = Command::new("nordvpn")
        .arg("countries")
        .output()
        .expect("Err...");

    return str_to_vec(parse_output(output), ", ".to_string())
        .iter()
        .map(|it| Country {
            name: clean_string(it),
            cities: get_cities(it),
        })
        .collect();
}

fn get_cities(country: &str) -> Vec<City> {
    let output = Command::new("nordvpn")
        .arg("cities")
        .arg(country)
        .output()
        .expect("Err...");

    return str_to_vec(parse_output(output), ", ".to_string())
        .iter()
        .map(|it| City {
            name: clean_string(it),
        })
        .collect();
}

pub fn get_status() -> Status {
    let output = Command::new("nordvpn")
        .arg("status")
        .output()
        .expect("Err...");

    let result = parse_output(output);
    return Status {
        status: extract_string(&result, "Status: "),
        ip: extract_string(&result, "IP: "),
        country: extract_string(&result, "Country: "),
        city: extract_string(&result, "City: "),
        transfer: extract_transfer(extract_string(&result, "Transfer: ").replace(", ", "\n")),
        uptime: extract_string(&result, "Uptime: ")
            .replace(" hours ", ":")
            .replace(" minutes ", ":")
            .replace(" seconds", ""),
    };
}

fn extract_transfer(result: String) -> Transfer {
    Transfer {
        up: extract_string(&result, " sent"),
        down: extract_string(&result, " received"),
    }
}

fn extract_string(source: &str, arg: &str) -> String {
    let result = source
        .lines()
        .find(|it| it.contains(arg))
        .get_or_insert("")
        .to_string()
        .replace(arg, "");
    return clean_string(&result);
}

fn parse_output(cmd_output: Output) -> String {
    let result = String::from_utf8_lossy(&cmd_output.stdout).to_string();

    if result.contains("NordVPN") {
        let result_arr: Vec<String> = result.split(".").map(|s| s.to_string()).collect();
        return result_arr[1].to_string();
    }

    return result;
}

fn str_to_vec(input: String, separator: String) -> Vec<String> {
    return input
        .to_string()
        .split(&separator.to_string())
        .map(|s| s.to_string())
        .collect();
}

fn clean_string(input: &str) -> String {
    return input.replace("\r-\r  \r\r-\r  \r", "").replace("\n", "");
}

pub(crate) fn connect(val: &str) {
    Command::new("nordvpn")
        .arg("c")
        .arg(val)
        .output()
        .expect("Err...");
}

pub(crate) fn disconnect() {
    Command::new("nordvpn").arg("d").output().expect("Err...");
}
