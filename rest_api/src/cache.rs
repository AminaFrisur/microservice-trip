use chrono::{DateTime, Utc};
use anyhow::anyhow;
use std::sync::{Arc, Mutex};
#[path = "./circuitbreaker.rs"] mod circuitbreaker;
use crate::circuitbreaker::CircuitBreaker;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Booking_Response {
    buchungsNummer: i32,
    buchungsDatum: String,
    loginName: String,
    fahrzeugId: i32,
    dauerDerBuchung: String,
    preisNetto: f32,
    status: String
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Booking {
    buchungsNummer: i32,
    buchungsDatum: String,
    loginName: String,
    fahrzeugId: i32,
    dauerDerBuchung: i32,
    preisNetto: f32,
    status: String,
    cacheTimestamp: String
}

impl Booking {
    pub fn new(buchungsNummer: i32, buchungsDatum: String, loginName: String, dauerDerBuchung: i32, fahrzeugId: i32, preisNetto: f32, status: String) -> Self {

        return Self {buchungsNummer, loginName, dauerDerBuchung,
                    buchungsDatum, fahrzeugId, preisNetto,
                     status, cacheTimestamp: Utc::now().to_rfc3339()};
    }

    pub fn set_status(&mut self, status: String) { self.status = status; }

}

#[derive(Clone)]
pub struct Cache {
    cached_bookings: Arc<std::sync::Mutex<Vec<Booking>>> ,
    max_size: i64,
    cache_time: i64
}
impl Cache   {
    pub fn new(max_size: i64, cache_time: i64) -> Self {

        return Self {cached_bookings: Arc::new(Mutex::new(Vec::new())), max_size, cache_time};
    }

    fn get_login_name(&self, index: usize) -> String {
        let cached_bookings = self.cached_bookings.lock().unwrap();
        let s: String = format!("{}", cached_bookings[index].loginName);
        return s;

    }

    fn clear_cache(& mut self) -> Result<(), anyhow::Error> {
        println!("Cache: Prüfe ob Einträge aus dem Cache gelöscht werden können");
        let mut cached_bookings = self.cached_bookings.lock().unwrap();

        if cached_bookings.len() > self.max_size.try_into().unwrap() {
            // kompletter reset des caches
            // sollte aber eigentlich nicht passieren
            *cached_bookings =  Vec::new();
            return Ok(());
        }

        let mut temp_index = cached_bookings.len();
        let mut check = true;
        let current_timestamp = Utc::now();

        while check {
            temp_index = temp_index / 2;
            println!("Cache: TempIndex ist {}", temp_index);
            // Falls im Cache nur ein Element ist
            if temp_index >= 1 {

                let cached_booking_timestamp = DateTime::parse_from_rfc3339(&cached_bookings[temp_index - 1].cacheTimestamp)?.with_timezone(&Utc);

                let time_diff = current_timestamp.signed_duration_since(cached_booking_timestamp).num_seconds();

                println!("Cache: Zeit Differenz zwsichen Aktueller Zeit und Cachetime beträgt {} Sekunden", time_diff - self.cache_time);
                // Wenn für den Eintrag die Cache Time erreicht ist -> lösche die hälfte vom Part des Arrays was betrachtet wird
                // Damit sind dann nicht alle alten Cache einträge gelöscht -> aber das clearen vom Cache sollte schnell gehen
                if time_diff >= self.cache_time {
                    println!("Cache: Clear Cache");
                    *cached_bookings = cached_bookings[temp_index..].to_vec();
                    check = false;
                }

                // Wenn timeDiff noch stimmt dann mache weiter

            } else {

                // auch wenn das eine Element im Array ein alter Eintrag ist
                // kann dies vernachlässigt werden bzw. ist nicht so wichtig
                println!("Cache: nichts zu clearen");
                check = false;
            }
        }

        Ok(())
    }

    pub fn get_cache_entry_index(& mut self, buchungsNummer: i32) -> Result<usize, anyhow::Error> {
        self.clear_cache()?;
        let mut final_index: usize = 0;
        let mut cached_bookings = self.cached_bookings.lock().unwrap();
        let mut booking_found = false;

        for i in 0..(cached_bookings.len()) {
            println!("{}", cached_bookings[i].buchungsNummer);
            if cached_bookings[i].buchungsNummer == buchungsNummer {
                final_index = i;
                // Auch beim Suchen eines Users -> Timestamp für Cache Eintrag aktualisieren
                println!("Cache: Update Timestamp vom Cache Eintrag");
                cached_bookings[i].cacheTimestamp = Utc::now().to_rfc3339();
                booking_found = true;
                break;
            }
        }

        if booking_found {
            Ok(final_index)
        } else {
            Err(anyhow!("Cache: Buchung wurde im Cache nicht gefunden"))
        }

    }

    pub fn update_or_insert_cached_entrie(&mut self, booking_found: bool, index: usize, newCacheEntry: Booking) -> Result<(), anyhow::Error> {
        let mut cached_bookings = self.cached_bookings.lock().unwrap();

        if booking_found {
            println!("Booking Cache: mache ein Update");
            cached_bookings.remove(index);
        }
        // Füge User neu im Cache hinzu, da nicht im cache vorhanden
        println!("Booking Cach: Füge Eintrag neu in Cache hinzu");
        cached_bookings.push(newCacheEntry);

        Ok(())
    }


    pub fn remove_from_cache(&mut self, index: usize) {
        let mut cached_bookings = self.cached_bookings.lock().unwrap();
        println!("Booking Cache: Lösche Buchung aus dem Cache");
        cached_bookings.remove(index);
    }

    pub fn get_all_cache_entrys(&self) -> Result<Vec<Booking>, anyhow::Error> {
        let cached_bookings = self.cached_bookings.lock().unwrap();
        Ok(cached_bookings.clone())
    }

    pub fn get_booking_from_cache(&self, index: usize) -> Booking {
        let cached_bookings = self.cached_bookings.lock().unwrap();
        cached_bookings[index].clone()
    }


    pub async fn check_and_get_booking_in_cache(&mut self, login_name: &str, auth_token: &str, buchungsnummer: i32, circuit_breaker: &mut CircuitBreaker<'_>) -> Result<(Booking, bool, usize ), anyhow::Error> {
        let mut booking_found = false;

        // Schritt 1: Prüfe ob Buchung aktuell im Cache befindet
        let booking_index = match self.get_cache_entry_index(buchungsnummer) {
            Ok(index) => {
                booking_found = true;
                index
            },
            Err(_) => {
                0
            }
        };
        // Wenn Ja gebe Buchung direkt aus dem Cache zuück
        if booking_found {
            let found_booking_login_name = self.get_login_name(booking_index);
            if login_name != found_booking_login_name {
                // let cached_bookings = self.cached_bookings.unlock().unwrap();
                println!("Cache Booking: übergebener LoginName entpricht nicht dem aus dem Cache");
                println!("Cache Booking: Zugriff auf die Buchung ist nicht erlaubt");
                return Err(anyhow!("Zugriff auf die Buchung nicht erlaubt!"));
            } else {

                return Ok((self.get_booking_from_cache(booking_index), booking_found, booking_index));
            }
        } else {
            // Wenn nein: Buchung ist nicht im cache
            // Also mache einen Request auf den Microservice Buchungsverwaltung
            println!("BookingCache: HeaderDaten = {} und  {}", login_name, auth_token);
            let addr_with_params = format!("/getBooking/{}", buchungsnummer);
            println!("{}", addr_with_params);

            match circuit_breaker.circuit_breaker_post_request(addr_with_params, login_name.to_string(), auth_token.to_string(), "GET".to_string()).await {
                Ok((_, response_json_string)) =>  {

                    // Serialisiere den Reponse und wandel dann in Booking um
                    let mut s = response_json_string.replace("[", "");
                    s = s.replace("]", "");
                    println!("{}", s);
                    let current_booking: Booking_Response = serde_json::from_str(&s)?;
                    let dauerDerBuchung: i32 = current_booking.dauerDerBuchung.parse()?;
                    let cacheBooking: Booking = Booking::new(current_booking.buchungsNummer, current_booking.buchungsDatum,
                                                             current_booking.loginName, dauerDerBuchung, current_booking.fahrzeugId,
                                                             current_booking.preisNetto, current_booking.status);

                    return Ok((cacheBooking, booking_found, 0));
                },
                Err(err) => return Err(err)
            }

        }

    }

}


