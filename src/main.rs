/* 
 * Copyright (C) Alexander Chuprynov <achuprynov@gmail.com>
 * This file is part of solution of test task described in readme.txt.
 */
// #![allow(unused_variables)]
// #![allow(unused_imports)]
// #![allow(dead_code)]

extern crate iron;
extern crate urlencoded;
extern crate serde;
extern crate router;
extern crate reqwest;

#[macro_use]
extern crate serde_json;

use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedQuery;
use router::Router;
use serde_json::Value;

fn main() {
    start_service("localhost:3000");
}

fn start_service(address: &str) {
   let mut router = Router::new();

    router.get("/", service_info, "");
    router.get("/:query", weather_forecast, "weather_forecast");

    let _server = Iron::new(router).http(address).unwrap();

    println!("Service started on {}\n", address);
}

fn service_info(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Example: http://localhost:3000/weather_forecast?city=Moscow,ru&day=Today")))
}

fn weather_forecast(request: &mut Request) -> IronResult<Response> {
    match get_forecast(request) {
        Ok(ref response) => Ok(Response::with((status::Ok, "application/json", response.to_string()))),
        Err(ref error) => Ok(Response::with((status::BadRequest, "application/json", error.to_string())))
    }
}

fn get_forecast(request: &mut Request) -> Result<Value, Value> {
    let (city, day) = match get_and_check_params(request) {
        Ok((city, day)) => (city, day),
        Err(ref error) => return Err(json!({"error" : error}))
    };

    let data1 = match get_forecast_from_apixu(&city) {
        Ok(data1) => data1,
        Err(ref error) => return Err(json!({"error" : error}))
    };

    let data2 = match get_forecast_from_yahoo(&city) {
        Ok(data2) => data2,
        Err(ref error) => return Err(json!({"error" : error}))
    };

    let forecast = get_average_forecast(&day, &data1, &data2);

    Ok(json!({
        "city" : city,
        "temperature_unit" : "C",
        "forecast" : forecast
    }))
}

fn get_and_check_params(request: &mut Request) -> Result<(String, String), String> {
    match request.get_ref::<UrlEncodedQuery>() {
        Ok(ref hashmap) => {
            let day = match hashmap.get("day") {
                Some(day) => day.get(0).unwrap().to_string(),
                None => return Err("Missing parameter 'day'".to_string())
            };

            match day.as_ref() {
                "Today" | "Tomorrow" | "5day" => (),
                _ => return Err("Incorrect 'day'. Available: 'Today', 'Tomorrow', '5day'.".to_string())
            }

            let city = match hashmap.get("city") {
                Some(city) => city.get(0).unwrap().to_string(),
                None => return Err("Missing parameter 'city'".to_string())
            };

            // println!("city = {:?}, day = {:?}\n", city, day);

            Ok((city, day))
        },
        Err(ref error) => return Err(error.to_string())
    }
}

fn get_forecast_from_apixu(city: &str) -> Result<Vec<(String, f64, f64)>, String> {
    let apikey = "";

    let url = format!(
        "http://api.apixu.com/v1/forecast.json?key={}&days=5&q={}",
        apikey,
        city);

    let json = try!{do_request(&url)};

    let mut result: Vec<(String, f64, f64)> = Vec::new();

    for day in 0..5 {
        let item = &json["forecast"]["forecastday"][day];

        result.push((
            item["date"].as_str().unwrap().to_string(),
            item["day"]["mintemp_f"].as_f64().unwrap(),
            item["day"]["maxtemp_f"].as_f64().unwrap()
        ));
    }

    // println!("apixu {:?}\n", &result);

    Ok(result)
}

fn get_forecast_from_yahoo(city: &str) -> Result<Vec<(String, f64, f64)>, String> {
    let url = format!(        
        "https://query.yahooapis.com/v1/public/yql?q=select%20*%20from%20weather.forecast%20where%20woeid%20in%20(select%20woeid%20from%20geo.places(1)%20where%20text%3D%22{}%22)&format=json&env=store%3A%2F%2Fdatatables.org%2Falltableswithkeys", 
        city);

    let json = try!{do_request(&url)};

    let mut result: Vec<(String, f64, f64)> = Vec::new();

    for day in 0..5 {
        let item = &json["query"]["results"]["channel"]["item"]["forecast"][day];

        result.push((
            item["date"].as_str().unwrap().to_string(),
            item["low"].as_str().unwrap().parse::<f64>().unwrap(),
            item["high"].as_str().unwrap().parse::<f64>().unwrap()
        ));
    }

    // println!("yahoo {:?}\n", &result);

    Ok(result)
}

fn do_request(request: &str) -> Result<serde_json::Value, String> {
    match reqwest::get(request) {
        Ok(ref mut response) => {
            match response.status().is_success() {
                true => {
                    match serde_json::from_str(&response.text().unwrap()) {
                        Ok(json) => Ok(json),
                        Err(error) => Err(error.to_string())
                    }
                },
                false => Err(response.status().to_string())
            }
        },
        Err(error) => Err(error.to_string())
    }
}

fn get_average_forecast(day: &str, data1: &Vec<(String, f64, f64)>, data2: &Vec<(String, f64, f64)>) -> Vec<Value> {
    let today_index = 0;
    let tomorrow_index = 1;
    let mut result: Vec<Value> = Vec::new();

    match day {
        "Today"    => result.push(get_average_for_one_day(&data1[today_index], &data2[today_index])),
        "Tomorrow" => result.push(get_average_for_one_day(&data1[tomorrow_index], &data2[tomorrow_index])),
        "5day" => for day in 0..5 { result.push(get_average_for_one_day(&data1[day], &data2[day])); }
        _ => ()
    }

    result
}

fn get_average_for_one_day(data1: &(String, f64, f64), data2: &(String, f64, f64)) -> Value {
    json!({
        "date" : data1.0,
        "temp_min" : convert_fahrenheit_to_celsius((data1.1 + data2.1)/2.0),
        "temp_max" : convert_fahrenheit_to_celsius((data1.2 + data2.2)/2.0) 
    })
}

fn convert_fahrenheit_to_celsius(temp: f64) -> i32 {
    // solution from https://github.com/elliotekj/learning-rust/tree/master/fahrenheit_to_celsius
    return ((temp - 32.0) * 5.0/9.0) as i32;
}