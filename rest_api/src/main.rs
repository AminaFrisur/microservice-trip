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


// Structs for Body Parsen
#[derive(Serialize, Deserialize, Debug)]
pub struct Vehicle_Location {
    buchungsNummer: i32,
    langtitude: i64,
    longitude: i64
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Vehicle_Command {
    buchungsNummer: i32,
    kommando: String
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

async fn handle_request(mut cache: Cache, mut circuit_breaker: CircuitBreaker<'_>, req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {

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
                Err(err) => return Ok(response_build(&format!("Authentifizierung fehlgeschlagen: {}", err), 401)),
            }

            match cache.get_all_cache_entrys() {
                Ok(cacheEntrys) => Ok(response_build(&serde_json::to_string(&cacheEntrys)?, 200 )),
                Err(err) => return Ok(response_build(&format!("Authentifizierung fehlgeschlagen: {}", err), 500)),
            }
        }

        (&Method::POST, "/updateVehicleLocation") => {
            let cloned_login_name = login_name.to_string();
            let cloned_auth_token = auth_token.to_string();
            // hyper::body::to_bytes verändert anscheinend req
            // deshalb muss login_name und auth_token werte kopiert werden
            // damit diese nicht durch die Method verändert werden
            let byte_stream = hyper::body::to_bytes(req).await?;
            match auth::check_auth_user(&cloned_login_name[..], &cloned_auth_token[..], false, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("Authentifizierung fehlgeschlagen: {}", err), 401)),
            }

            let location: Vehicle_Location = serde_json::from_slice(&byte_stream)?;
            let mut booking_data = cache.check_and_get_booking_in_cache(&cloned_login_name[..], &cloned_auth_token[..], location.buchungsNummer, &mut circuit_breaker).await?;
            let status = booking_data.0.get_status();
            if status == "started".to_string() {
                booking_data.0.set_longitude(location.longitude);
                booking_data.0.set_langtitude(location.langtitude);
                cache.update_or_insert_cached_entrie(booking_data.1, booking_data.2, booking_data.0)?;
                Ok(response_build("Fahrzeug Standort wurde aktualisiert",200))
            } else {
                Ok(response_build("Buchung konnte unter angegebener Buchungsnummer und Nutzername nicht gefunden werden !", 500))
            }
        }

        (&Method::POST, "/sendVehicleCommand") => {

            let cloned_login_name = login_name.to_string();
            let cloned_auth_token = auth_token.to_string();
            // hyper::body::to_bytes verändert anscheinend req
            // deshalb muss login_name und auth_token werte kopiert werden
            // damit diese nicht durch die Method verändert werden
            let byte_stream = hyper::body::to_bytes(req).await?;
            match auth::check_auth_user(&cloned_login_name[..], &cloned_auth_token[..], false, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("Authentifizierung fehlgeschlagen: {}", err), 401)),
            }

            let command: Vehicle_Command = serde_json::from_slice(&byte_stream)?;
            let mut booking_data = cache.check_and_get_booking_in_cache(&cloned_login_name[..], &cloned_auth_token[..], command.buchungsNummer, &mut circuit_breaker).await?;
            let status = booking_data.0.get_status();
            if status == "started".to_string() {
                cache.update_or_insert_cached_entrie(booking_data.1, booking_data.2, booking_data.0)?;
                // TODO: Mockup Request zu Fahrzeug
                Ok(response_build("Fahrzeug Kommando ausgeführt",200))
            } else {
                Ok(response_build("Buchung konnte unter angegebener Buchungsnummer und Nutzername nicht gefunden werden !", 500))
            }
        }


        (&Method::POST, "/startTrip/") => {

            match auth::check_auth_user(login_name, auth_token, false, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("Authentifizierung fehlgeschlagen: {}", err), 401)),
            }

            // extrahiere die Buchungsnummer aus der URL
            let buchungsnummer_string: String = regex_route.chars().filter(|c| c.is_digit(10)).collect();
            let buchungsnummer: i32 = buchungsnummer_string.parse()?;

            let addr_with_params: String = format!("/startTrip/{}", buchungsnummer);

            let mut booking_data = cache.check_and_get_booking_in_cache(login_name, auth_token, buchungsnummer, &mut circuit_breaker).await?;

            // rufe CircuitBreaker auf
            match circuit_breaker.circuit_breaker_post_request(addr_with_params, login_name.to_string(), auth_token.to_string(), "POST".to_string()).await {
                Ok(res) =>  {
                    booking_data.0.set_status("started".to_string());
                    cache.update_or_insert_cached_entrie(booking_data.1, booking_data.2, booking_data.0)?;
                    Ok(response_build("Trip wurde gestartet", 200))
                },
                Err(err) => return  Ok(response_build("Trip konnte nicht gestartet werden", 401)),
            }
        }

        (&Method::POST, "/endTrip/") => {

            match auth::check_auth_user(login_name, auth_token, false, JWT_SECRET).await {
                Ok(()) => println!("Rest API: Nutzer ist authentifiziert"),
                Err(err) => return Ok(response_build(&format!("Authentifizierung fehlgeschlagen: {}", err), 401)),
            }

            // extrahiere die Buchungsnummer aus der URL
            let buchungsnummer_string: String = regex_route.chars().filter(|c| c.is_digit(10)).collect();
            let buchungsnummer: i32 = buchungsnummer_string.parse()?;

            let addr_with_params: String = format!("/endTrip/{}", buchungsnummer);

            // rufe CircuitBreaker auf
            match circuit_breaker.circuit_breaker_post_request(addr_with_params, login_name.to_string(), auth_token.to_string(), "POST".to_string()).await {
                Ok(res) =>  {
                    // Prüfe ob Buchung im Cache ist
                    // Wenn ja -> Lösche die Buchung aus dem Cache, da Trip beendet
                    match cache.get_cache_entry_index(buchungsnummer) {
                        Ok(index) => {
                            cache.remove_from_cache(index);
                            println!("ENDTRIP: Buchung befindet sich im Cache -> Lösche Buchung aus dem Cache!")
                        },
                        Err(_) => {
                            println!("ENDTRIP: Buchung befindet sich nicht im Cache");
                        }
                    };

                    Ok(response_build("Trip wurde beendet", 200))
                },
                Err(err) => return  Ok(response_build("Trip konnte nicht beendet werden", 401)),
            }
        }

        _ => {
            println!("REST API: ROUTE NOT FOUND");
            Ok(response_build("Route not found", 404))
        }
    }
}

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


