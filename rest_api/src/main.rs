use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::result::Result;
use serde::{Deserialize, Serialize};
extern crate regex;
use regex::Regex;
mod cache;
mod circuitbreaker;
mod auth;
use crate::circuitbreaker::CircuitBreaker;
use crate::cache::Cache;
use crate::cache::Booking;

#[derive(Serialize, Deserialize, Debug)]
struct Buchung {
    buchungsNummer: i32,
    buchungsDatum: String,
    loginName: String,
    fahrzeugId: i32,
    preisNetto: f32,
    status: String,
    cacheTimestamp: f64
}

impl Buchung {
    fn new(
        buchungsNummer: i32,
        buchungsDatum: String,
        loginName: String,
        fahrzeugId: i32,
        preisNetto: f32,
        status: String,
        cacheTimestamp: f64
    ) -> Self {
        Self {
            buchungsNummer,
            buchungsDatum,
            loginName,
            fahrzeugId,
            preisNetto,
            status,
            cacheTimestamp
        }
    }
}

pub fn regex_route(re: Regex, route: &str) -> String {
    if re.is_match(route) {
        let cap = re.captures(route).unwrap();
        return (&cap[0]).to_string();
    } else {
        return "/error".to_string();
    }
}

async fn handle_request_wrapper(cache: Cache, circuit_breaker: CircuitBreaker<'_>, req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    match handle_request(cache, circuit_breaker, req).await {
        Ok(result) => Ok(result),
        Err(err) => {
            let error_message = format!("{:?}", err);
            Ok(response_build(&error_message, 500))

        }
    }
}

async fn handle_request(cache: Cache, circuit_breaker: CircuitBreaker<'_>, req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {

    let mut login_name ="";
    let mut auth_token ="";
    let JWT_SECRET : String = "goK!pusp6ThEdURUtRenOwUhAsWUCLheasfr43qrf43rttq3".to_string();

    // get Header Information for login_name and auth_token
    for (key, value) in req.headers().iter() {
        if key == "login_name" {
            login_name = value.to_str()?;
            // login_name = &value;
            println!("REST API login_name found {:?}", login_name);

        }
        if key == "auth_token" {
            auth_token = value.to_str()?;
            // auth_token = &value;
            println!("REST API auth_token found {:?}", auth_token);

        }
    }

    // Definiere hier zusätlich welche Routen erlaubt sind
    // Wichtig um auch zu checken ob Parameter in der URL dabei sind
    let re = Regex::new(r"/startTrip/\d+|/endTrip/\d+|/getAllRunningTrips|/sendVehicleCommand|/updateVehicleLocation")?;
    let regex_route = regex_route(re, req.uri().path());
    let filtered_route: String = regex_route.chars().filter(|c| !c.is_digit(10)).collect();

    match (req.method(),  filtered_route.as_str()) {

        (&Method::GET, "/getAllRunningTrips") => {

            match auth::check_auth_user(login_name, auth_token, true, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("{}", err), 401)),
            }

            match cache.get_all_cache_entrys() {
                Ok(cacheEntrys) => Ok(response_build(&serde_json::to_string(&cacheEntrys)?, 200 )),
                Err(err) => return Ok(response_build(&format!("{}", err), 500)),
            }
        }


        (&Method::POST, "/updateVehicleLocation") => {

            match auth::check_auth_user(login_name, auth_token, false, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("{}", err), 401)),
            }

            let byte_stream = hyper::body::to_bytes(req).await?;
            //let fahrzeug: Fahrzeug = serde_json::from_slice(&byte_stream)?;

            Ok(response_build("Fahrzeug Standort wurde aktualisiert",200))
        }

        (&Method::POST, "/sendVehicleCommand") => {

            match auth::check_auth_user(login_name, auth_token, false, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("{}", err), 401)),
            }


            let byte_stream = hyper::body::to_bytes(req).await?;


            Ok(response_build("Fahrzeug Kommando ausgeführt", 200))
        }

        (&Method::POST, "/startTrip/") => {

            match auth::check_auth_user(login_name, auth_token, false, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("{}", err), 401)),
            }




            Ok(response_build("Trip wurde gestartet", 200))
        }

        (&Method::POST, "/endTrip/") => {

            match auth::check_auth_user(login_name, auth_token, false, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("{}", err), 401)),
            }


            Ok(response_build("Trip wurde beendet", 200))
        }

        _ => {
            println!("REST API: ROUTE NOT FOUND");
            Ok(response_build("Route not found", 404))
        }
    }
}

// TODO: Prüfe ob wirklich gebraucht wird
// CORS headers
fn response_build(body: &str, status: u16) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(body.to_owned()))
        .unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let circuit_breaker_buchungsverwaltung = CircuitBreaker::new(150, 30, 0, -3, 10, 3, "api-gateway-buchungsverwaltung", 80);
    let cache_booking = Cache::new(10000, 10000);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let make_svc = make_service_fn(|_| {
        let circuit_breaker_buchungsverwaltung = circuit_breaker_buchungsverwaltung.clone();
        let cache_booking = cache_booking.clone();
        // move converts any variables captured by reference or mutable reference to variables captured by value.
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let circuit_breaker_buchungsverwaltung = circuit_breaker_buchungsverwaltung.clone();
                let cache_booking = cache_booking.clone();
                handle_request_wrapper(cache_booking, circuit_breaker_buchungsverwaltung, req)
            }))
        }
    });
    println!("REST API: Start Server");
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}


