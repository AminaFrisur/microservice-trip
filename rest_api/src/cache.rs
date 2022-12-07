use chrono::{DateTime, Utc};
use anyhow::anyhow;
use std::sync::{Arc, Mutex};
#[path = "./circuitBreaker.rs"] mod circuit_breaker;

#[derive(Clone)]
pub struct Booking {
    buchungsNummer: i32,
    buchungsDatum: String,
    loginName: String,
    fahrzeugId: i32,
    preisNetto: f32,
    status: String,
    cacheTimestamp: f64
}

impl Booking {
    pub fn new(buchungsNummer: &str, buchungsDatum: &str, loginName: DateTime<Utc>, fahrzeugId: i32, preisNetto: f32, status: &str, cacheTimestamp: f64) -> Self {

        return Self {buchungsNummer: buchungsNummer.to_string(), buchungsDatum: buchungsDatum.to_string(),
                     loginName: loginName.to_rfc3339(), fahrzeugId,preisNetto,
                     status: status.to_string(),cacheTimestamp};
    }

    pub fn print_login_name(&self) {
        println!("CACHE LOGIN NAME: {}", self.login_name);
    }

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

                let cached_booking_timestamp = DateTime::parse_from_rfc3339(&cached_bookings[temp_index - 1].cache_timestamp)?.with_timezone(&Utc);

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
        let mut booking_found: bool = false;
        let mut cached_bookings = self.cached_bookings.lock().unwrap();
        let mut booking_found = false;

        for i in 0..(cached_bookings.len()) {
            println!("{}", cached_bookings[i].buchungsNummer);
            if cached_user[i].buchungsNummer == buchungsNummer {
                final_index = i;
                // Auch beim Suchen eines Users -> Timestamp für Cache Eintrag aktualisieren
                println!("Cache: Update Timestamp vom Cache Eintrag");
                cached_user[i].cache_timestamp = Utc::now().to_rfc3339();
                booking_found = true;
                break;
            }
        }

        if booking_found {
            println!("Cache: Buchung Index ist: {} ", final_index);
            Ok(final_index)
        } else {
            Err(anyhow!("Cache: Buchung wurde im Cache nicht gefunden"))
        }

    }

    pub fn update_or_insert_cached_entrie(&self, index: usize, newCacheEntry: Booking) -> Result<(), anyhow::Error> {
        let mut cached_bookings = self.cached_bookings.lock().unwrap();

        if index >= 0  {
            println!("Booking Cache: mache ein Update");
            cached_bookings.remove(index);
        }
        // Füge User neu im Cache hinzu, da nicht im cache vorhanden
        println!("Booking Cach: Füge Eintrag neu in Cache hinzu");
        cached_user.push(Booking);

        Ok(())
    }


    pub fn remove_from_cache(&self, index: usize) -> Result<(), anyhow::Error> {
        let mut cached_bookings = self.cached_bookings.lock().unwrap();

        if index >= 0  {
            println!("Booking Cache: Lösche Buchung aus dem Cache");
            cached_bookings.remove(index);
        }

        Ok(())
    }

    pub fn get_all_cache_entrys(&self) -> Result<(Arc<std::sync::Mutex<Vec<Booking>>>), anyhow::Error> {
        let mut cached_bookings = self.cached_bookings.lock().unwrap();
        Ok((cached_bookings))
    }


    pub fn check_and_get_booking_in_cache(&self, login_name: &str, buchungsnummer: i32, circuit_breaker: circuit_breaker) -> Result<(Booking, usize ),anyhow::Error> {
        let cached_bookings = self.cached_bookings.lock().unwrap();
        let index = self.get_cache_entry_index(buchungsnummer);
        if index >= 0 {
            if login_name != self.cached_bookings[index].login_name {
                println!("Cache Booking: übergebener LoginName entpricht nicht dem aus dem Cache");
                println!("Cache Booking: Zugriff auf die Buchung ist nicht erlaubt");
                Err("Zugriff auf Buchung ist vom aktuellen Nutzer nicht erlaubt")
            } else {
                Ok((self.cached_bookings[index], index))
            }
        } else {
            let headerData = {

            };
        }


        // TODO:
        Ok(true)

    }

}


