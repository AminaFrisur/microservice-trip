use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use wasm_bindgen::prelude::*;
use std::str::FromStr;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: String);
}

// Nutze den Cirucit Breaker aus Javascript
#[wasm_bindgen(module = "/circuitBreaker.js")]
extern "C" {
    pub type CircuitBreaker;

    #[wasm_bindgen(method, catch)]
    pub async fn circuitBreakerRequestForWasmRust(this: &CircuitBreaker, path: String, bodyData: String, HeaderData: String, httpMethod: String) -> Result<JsValue,JsValue>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Booking_Response {
    buchungsNummer: i32,
    buchungsDatum: String,
    loginName: String,
    fahrzeugId: i32,
    dauerDerBuchung: String,
    preisNetto: f32,
    status: String,
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Booking {
    buchungsNummer: i32,
    buchungsDatum: String,
    loginName: String,
    fahrzeugId: i32,
    dauerDerBuchung: i32,
    preisNetto: f32,
    status: String,
    cacheTimestamp: String,
    longitude: i64,
    langtitude: i64
}

#[wasm_bindgen]
impl Booking {
    #[wasm_bindgen(constructor)]
    pub fn new(buchungsNummer: i32, buchungsDatum: String, loginName: String, dauerDerBuchung: i32, fahrzeugId: i32, preisNetto: f32,
               status: String, longitude: i64, langtitude: i64) -> Self {

        return Self {buchungsNummer, loginName, dauerDerBuchung,
            buchungsDatum, fahrzeugId, preisNetto,
            status, cacheTimestamp: Utc::now().to_rfc3339(),
            longitude, langtitude };
    }

    #[wasm_bindgen(method)]
    pub fn get_buchungsNummer(&self) -> i32 { self.buchungsNummer.into() }

    #[wasm_bindgen(method)]
    pub fn get_buchungsDatum(&self) -> String { self.buchungsDatum.clone() }

    #[wasm_bindgen(method)]
    pub fn get_loginName(&self) -> String { self.loginName.clone() }

    #[wasm_bindgen(method)]
    pub fn get_fahrzeugId(&self) -> i32 { self.fahrzeugId.into() }

    #[wasm_bindgen(method)]
    pub fn get_dauerDerBuchung(&self) -> i32 { self.dauerDerBuchung.into() }

    #[wasm_bindgen(method)]
    pub fn get_preisNetto(&self) -> f32 { self.preisNetto.into() }

    #[wasm_bindgen(method)]
    pub fn get_status(&self) -> String { self.status.clone() }

    #[wasm_bindgen(method)]
    pub fn get_longitude(&self) -> i64 { self.buchungsNummer.into() }

    #[wasm_bindgen(method)]
    pub fn get_langtitude(&self) -> i64 { self.buchungsNummer.into() }

    #[wasm_bindgen(method)]
    pub fn set_status(&mut self, status: String) { self.status = status; }

    #[wasm_bindgen(method)]
    pub fn set_longitude(&mut self, longitude: i64) { self.longitude = longitude; }

    #[wasm_bindgen(method)]
    pub fn set_langtitude(&mut self, langtitude: i64) { self.langtitude = langtitude; }

}
#[wasm_bindgen]
#[derive(Clone)]
pub struct Cache {
    cached_bookings: Arc<std::sync::Mutex<Vec<Booking>>> ,
    max_size: i64,
    cache_time: i64
}

#[wasm_bindgen]
impl Cache   {
    #[wasm_bindgen(constructor)]
    pub fn new(max_size: i64, cache_time: i64) -> Self {

        return Self {cached_bookings: Arc::new(Mutex::new(Vec::new())), max_size, cache_time};
    }

    fn get_login_name(&self, index: usize) -> String {
        let cached_bookings = self.cached_bookings.lock().unwrap();
        let s: String = format!("{}", cached_bookings[index].loginName);
        return s;

    }

    fn clear_cache(& mut self) {
        log("Cache: Prüfe ob Einträge aus dem Cache gelöscht werden können".to_string());
        let mut cached_bookings = self.cached_bookings.lock().unwrap();

        if cached_bookings.len() > self.max_size.try_into().unwrap() {
            // kompletter reset des caches
            // sollte aber eigentlich nicht passieren
            *cached_bookings =  Vec::new();
            return;
        }

        let mut temp_index = cached_bookings.len();
        let mut check = true;
        let current_timestamp = Utc::now();

        while check {
            temp_index = temp_index / 2;
            log(format!("Cache: TempIndex ist {}", temp_index));
            // Falls im Cache nur ein Element ist
            if temp_index >= 1 {

                let cached_booking_timestamp = DateTime::parse_from_rfc3339(&cached_bookings[temp_index - 1].cacheTimestamp).unwrap().with_timezone(&Utc);

                let time_diff = current_timestamp.signed_duration_since(cached_booking_timestamp).num_seconds();

                log(format!("Cache: Zeit Differenz zwsichen Aktueller Zeit und Cachetime beträgt {} Sekunden", time_diff - self.cache_time));
                // Wenn für den Eintrag die Cache Time erreicht ist -> lösche die hälfte vom Part des Arrays was betrachtet wird
                // Damit sind dann nicht alle alten Cache einträge gelöscht -> aber das clearen vom Cache sollte schnell gehen
                if time_diff >= self.cache_time {
                    log("Cache: Clear Cache".to_string());
                    *cached_bookings = cached_bookings[temp_index..].to_vec();
                    check = false;
                }

                // Wenn timeDiff noch stimmt dann mache weiter

            } else {

                // auch wenn das eine Element im Array ein alter Eintrag ist
                // kann dies vernachlässigt werden bzw. ist nicht so wichtig
                log("Cache: nichts zu clearen".to_string());
                check = false;
            }
        }
    }

    pub fn get_cache_entry_index(& mut self, buchungsNummer: i32) -> String {
        self.clear_cache();
        let mut final_index: usize = 0;
        let mut cached_bookings = self.cached_bookings.lock().unwrap();
        let mut booking_found = false;

        for i in 0..(cached_bookings.len()) {
            log(format!("{}", cached_bookings[i].buchungsNummer));
            if cached_bookings[i].buchungsNummer == buchungsNummer {
                final_index = i;
                // Auch beim Suchen eines Users -> Timestamp für Cache Eintrag aktualisieren
                log("Cache: Update Timestamp vom Cache Eintrag".to_string());
                cached_bookings[i].cacheTimestamp = Utc::now().to_rfc3339();
                booking_found = true;
                break;
            }
        }
        if booking_found {
            return format!("{}", final_index);
        } else {
            return format!("{}", -1);
        }

    }

    #[wasm_bindgen(method)]
    pub fn update_or_insert_cached_entrie(&mut self, booking_found: bool, index: usize, newCacheEntry: Booking) -> bool {

        let mut cached_bookings = self.cached_bookings.lock().unwrap();

        if booking_found {
            cached_bookings.remove(index);
        }
        cached_bookings.push(newCacheEntry);

        true
    }

    #[wasm_bindgen(method)]
    pub fn remove_from_cache(&mut self, index: usize) {
        let mut cached_bookings = self.cached_bookings.lock().unwrap();
        cached_bookings.remove(index);
    }

    //pub fn get_all_cache_entrys(&self) -> Vec<Booking> {
    //    let cached_bookings = self.cached_bookings.lock().unwrap();
    //    Ok(cached_bookings.clone())
    //}

    #[wasm_bindgen(method)]
    pub fn get_booking_from_cache(&self, index: usize) -> Booking {
        let cached_bookings = self.cached_bookings.lock().unwrap();
        cached_bookings[index].clone()
    }

    #[wasm_bindgen(method)]
    pub async fn check_and_get_booking_in_cache(&mut self, login_name: &str, auth_token: &str, buchungsnummer: i32, circuit_breaker: CircuitBreaker) -> Booking {
        // let mut booking_found = false;

        let default_booking_result = Booking {buchungsNummer: -1, buchungsDatum: "ERROR".to_string(),
                                              loginName: "ERROR".to_string(), fahrzeugId: -1, dauerDerBuchung: -1,
                                              preisNetto: -1.0, status: "ERROR".to_string(),  cacheTimestamp: "ERROR".to_string(),
                                              longitude: -1, langtitude: -1 };

        // Schritt 1: Prüfe ob Buchung aktuell im Cache befindet
        let booking_index = self.get_cache_entry_index(buchungsnummer);
        let booking_index_i32 = i32::from_str(&booking_index[..]).unwrap();

        // Wenn Ja gebe Buchung direkt aus dem Cache zuück
        if booking_index_i32 >= 0 {
            log("Cache Booking: Buchung wurde im Cache gefunden".to_string());
            let booking_index_usize = usize::from_str(&booking_index[..]).unwrap();
            let found_booking_login_name = self.get_login_name(booking_index_usize);
            if login_name != found_booking_login_name {
                // let cached_bookings = self.cached_bookings.unlock().unwrap();
                log("Cache Booking: übergebener LoginName entpricht nicht dem aus dem Cache".to_string());
                log("Cache Booking: Zugriff auf die Buchung ist nicht erlaubt".to_string());
                return default_booking_result;
            } else {
                log("Cache Booking: Zugriff auf Buchung erlaubt".to_string());
                return self.get_booking_from_cache(booking_index_usize);
            }
        } else {
            // Wenn nein: Buchung ist nicht im cache
            // Also mache einen Request auf den Microservice Buchungsverwaltung
            log(format!("BookingCache: HeaderDaten = {} und  {}", login_name, auth_token));
            let addr_with_params = format!("/getBooking/{}", buchungsnummer);
            log(format!("{}", addr_with_params));
            match circuit_breaker.circuitBreakerRequestForWasmRust(addr_with_params, login_name.to_string(), auth_token.to_string(), "GET".to_string()).await {
              Ok(res) => {
                  // Wandel bei Erfolg Response String in Booking Instanz um
                  let response_json = res.as_string();
                  let response_json_string;
                  response_json_string = match response_json {
                      Some(s) => s,
                      None => return default_booking_result,
                  };

                  // Warum auch immer wird bool rückgabe auch als String zurückgegeben
                  if response_json_string == "false".to_string() {

                      log(format!("CircuitBreaker Request schlug fehl. HTTP Code war != 200"));
                      return default_booking_result;
                  }

                  // Serialisiere den Reponse und wandel dann in Booking um
                  let mut response_json_string_formatted = response_json_string.replace("[", "");
                  response_json_string_formatted = response_json_string_formatted.replace("]", "");
                  log(format!("Formattierter JSON String ist {:?}", response_json_string_formatted));
                  log(format!("{}", response_json_string_formatted));
                  let current_booking: Booking_Response = match serde_json::from_str(&response_json_string_formatted) {
                      Ok(s) => s ,
                      Err(e) => {
                          log(format!("BookingCache: Kann Ergebniss nicht in Booking Instanz umwandeln wegen: {}", e));
                          return default_booking_result;
                      }
                  };
                  let dauerDerBuchung: i32 = current_booking.dauerDerBuchung.parse().unwrap();
                  let cacheBooking: Booking = Booking::new(current_booking.buchungsNummer, current_booking.buchungsDatum,
                                                           current_booking.loginName, dauerDerBuchung, current_booking.fahrzeugId,
                                                           current_booking.preisNetto, current_booking.status, 0, 0);

                  log(format!("BookingCache: CircuitBreaker Anfrage war erfolgreich {:?}", response_json_string_formatted));
                  return cacheBooking
              },

                Err(e) => {
                    log(format!("BookingCache: CircuitBreaker Anfrage an Buchungsverwaltung ist fehlgeschlagen"));
                    return default_booking_result
                }
            };

        }

    }

}