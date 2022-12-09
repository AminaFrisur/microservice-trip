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
    pub async fn circuitBreakerRequestForWasmRust(this: &CircuitBreaker, path: String, bodyData: String, HeaderData: String, httpMethod: String) -> Result<(),JsValue>;
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
    pub fn new(buchungsNummer: i32, buchungsDatum: String, loginName: String, dauerDerBuchung: i32, fahrzeugId: i32, preisNetto: f32, status: String) -> Self {

        return Self {buchungsNummer, loginName, dauerDerBuchung,
            buchungsDatum, fahrzeugId, preisNetto,
            status, cacheTimestamp: Utc::now().to_rfc3339(),
            longitude: 0, langtitude: 0 };
    }
    #[wasm_bindgen(method)]
    pub fn set_status(&mut self, status: String) { self.status = status; }
    #[wasm_bindgen(method)]
    pub fn get_status(&mut self) -> String { self.status.clone() }
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
        log("TEST 1".to_string());
        let mut final_index: usize = 0;
        log("TEST 2".to_string());
        let mut cached_bookings = self.cached_bookings.lock().unwrap();
        log("TEST 3".to_string());
        let mut booking_found = false;
        log("TEST 4".to_string());

        for i in 0..(cached_bookings.len()) {
            log("TEST 5".to_string());
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
        log("TEST 6".to_string());
        if booking_found {
            return format!("{}", final_index);
        } else {
            log("TEST 7".to_string());
            return format!("{}", -1);
        }

    }

    #[wasm_bindgen(method)]
    pub fn update_or_insert_cached_entrie(&mut self, booking_found: bool, index: usize, newCacheEntry: Booking) -> bool {

        let mut cached_bookings = self.cached_bookings.lock().unwrap();

        if booking_found {
            log("Booking Cache: mache ein Update".to_string());
            cached_bookings.remove(index);
        }
        // Füge User neu im Cache hinzu, da nicht im cache vorhanden
        log("Booking Cach: Füge Eintrag neu in Cache hinzu".to_string());
        cached_bookings.push(newCacheEntry);

        true
    }

    #[wasm_bindgen(method)]
    pub fn remove_from_cache(&mut self, index: usize) {
        let mut cached_bookings = self.cached_bookings.lock().unwrap();
        log("Booking Cache: Lösche Buchung aus dem Cache".to_string());
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
    pub async fn check_and_get_booking_in_cache(&mut self, login_name: &str, auth_token: &str, buchungsnummer: i32, circuit_breaker: CircuitBreaker) {
        let mut booking_found = false;
        log("TEST 8".to_string());
        // Schritt 1: Prüfe ob Buchung aktuell im Cache befindet
        let booking_index = self.get_cache_entry_index(buchungsnummer);
        let booking_index_i32 = i32::from_str(&booking_index[..]).unwrap();

        // Wenn Ja gebe Buchung direkt aus dem Cache zuück
        log("TEST 9".to_string());
        if booking_index_i32 >= 0 {
            let booking_index_usize = usize::from_str(&booking_index[..]).unwrap();
            let found_booking_login_name = self.get_login_name(booking_index_usize);
            if login_name != found_booking_login_name {
                // let cached_bookings = self.cached_bookings.unlock().unwrap();
                log("Cache Booking: übergebener LoginName entpricht nicht dem aus dem Cache".to_string());
                log("Cache Booking: Zugriff auf die Buchung ist nicht erlaubt".to_string());

            } else {


            }
        } else {
            // Wenn nein: Buchung ist nicht im cache
            // Also mache einen Request auf den Microservice Buchungsverwaltung
            log(format!("BookingCache: HeaderDaten = {} und  {}", login_name, auth_token));
            let addr_with_params = format!("/getBooking/{}", buchungsnummer);
            log(format!("{}", addr_with_params));
            log("TEST 10".to_string());
            let res = match circuit_breaker.circuitBreakerRequestForWasmRust(addr_with_params, login_name.to_string(), auth_token.to_string(), "GET".to_string()).await {
              Ok(e) => log("ALLES OK BEI CIRUCIT BREAKER".to_string()),
                Err(e) => log("ALLES NICHT OK BEI CIRUCIT BREAKER".to_string())
            };
            log(format!("{:?}", res));
            log("TEST 11".to_string());

        }

    }

}