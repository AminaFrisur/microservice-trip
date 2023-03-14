use chrono::{DateTime, Utc};
use anyhow::anyhow;
#[path = "./client.rs"] mod client;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CircuitBreaker<'a> {
    circuit_breaker_state: Arc<std::sync::Mutex<&'a str>>,
    success_count: Arc<std::sync::Mutex<i64>>,
    fail_count: Arc<std::sync::Mutex<i64>>,
    timeout_reset: i64,
    timeout_open_state: i64,
    trigger_half_state: i64,
    trigger_open_state: i64,
    trigger_closed_state: i64,
    timestamp:  Arc<std::sync::Mutex<DateTime<Utc>>>,
    permitted_requests_in_state_half: i64,
    request_count: Arc<std::sync::Mutex<i64>>,
    hostname:  &'a str,
    port: i32,
}
impl <'a> CircuitBreaker<'a>  {
    pub fn new(timeout_reset: i64, timeout_open_state: i64, trigger_half_state: i64, trigger_open_state: i64,
               permitted_requests_in_state_half: i64, trigger_closed_state: i64, hostname: &'a str, port: i32) -> Self {

        return Self { circuit_breaker_state: Arc::new(Mutex::new("CLOSED")), success_count: Arc::new(Mutex::new(0)), fail_count: Arc::new(Mutex::new(0)), timeout_reset, timeout_open_state,
            trigger_half_state, trigger_closed_state, timestamp: Arc::new(Mutex::new(Utc::now())), trigger_open_state, permitted_requests_in_state_half,
            request_count: Arc::new(Mutex::new(0)), hostname, port};
    }

    fn check_reset(&mut self, time_diff: i64) {
        println!("{}", time_diff);
        println!("{}", time_diff - self.timeout_reset);
        println!("Circuit Breaker: Prüfe ob Circuit Breaker Status zurückgesetzt werden soll");
        if time_diff > self.timeout_reset &&
            (self.get_circuit_breaker_state() == "CLOSED" || self.get_circuit_breaker_state() == "HALF") {
            println!("Circuit Breaker: Kompletter Status wird zurückgesetzt!");
            *(self.fail_count.lock().unwrap()) = 0;
            *(self.success_count.lock().unwrap()) = 0;
            *(self.timestamp.lock().unwrap()) = Utc::now();
            *(self.request_count.lock().unwrap()) = 0;
        }
    }

    pub async fn circuit_breaker_post_request(&mut self, addr_with_params: String, login_name: String, auth_token: String, http_method: String) -> Result<(http_req::response::Response, String), anyhow::Error> {

        println!("REST API: AKTUELLER CIRCUIT BREAKER STATUS IST: {}", self.get_circuit_breaker_state());

        let current_timestamp = Utc::now();
        let time_diff = current_timestamp.signed_duration_since(self.get_timestamp()).num_seconds();
        println!("timeDiff is {}", time_diff);
        println!("timeout_open_state ist {}", self.timeout_open_state);

        self.check_reset(time_diff);

        if self.get_circuit_breaker_state() == "OPEN" {

            if time_diff >= self.timeout_open_state {
                // Wenn timeout abgelaufen setze den Circuit Breaker wieder auf HALF
                println!("Circuit Breaker: Wechsel Circuit Breaker Status von OPEN auf HALF");
                self.set_circuit_breaker_state("HALF");

            } else {
                println!("Circuit Breaker: immer noch auf Zustand OPEN");
                println!("Circuit Breaker ist {} noch offen", (self.timeout_open_state - time_diff));
                return Err(anyhow!("Request fehlgeschlagen: Circuit Breaker ist im Zustand offen und keine Requests sind zum Service Benutzerverwaltung erlaubt"))
            }

        }

        if self.get_circuit_breaker_state() == "HALF" && self.get_request_count() > self.permitted_requests_in_state_half {
            println!("Request fehlgeschlagen: Circuit Breaker ist auf Zustand HALF aber der erlaubte RequestCount ist erreicht");
            return Err(anyhow!("Request fehlgeschlagen: Circuit Breaker ist auf Zustand HALF aber der erlaubte RequestCount ist erreicht"))
        }

        if self.get_circuit_breaker_state() == "HALF" && (self.get_success_count() - self.get_fail_count() > self.trigger_closed_state) {
            self.check_reset(time_diff);
            println!("Circuit Breaker: Wechsel Circuit Breaker Status von HALF auf CLOSED");
            self.set_circuit_breaker_state("CLOSED");
        }

        println!("Circuit Breaker: Führe HTTP Request im Circuit Breaker durch");
        if self.get_circuit_breaker_state() == "HALF" {
            self.increment_request_count();
        }

        match client::make_request(self.hostname, self.port, addr_with_params, login_name, auth_token, http_method).await {

            Ok((res, response_json_string)) => {
                self.increment_success_count();
                println!("Circuit Breaker: Request war erfolgreich. Success Count ist jetzt bei {}", self.get_success_count());

                if res.status_code().is_success() {
                    return Ok((res, response_json_string))
                } else {
                    return Err(anyhow!("CircuitBreaker: Request failed, return code was {}", res.status_code()))
                }

            },
            Err(err) => {
                println!("{:?}", err);

                self.increment_fail_count();
                if self.get_circuit_breaker_state() == "CLOSED" && self.get_success_count() - self.get_fail_count() < self.trigger_half_state {
                    println!("Circuit Breaker: Wechsel Circuit Breaker Status von CLOSED auf HALF");
                    self.set_circuit_breaker_state("HALF");
                    *(self.timestamp.lock().unwrap()) = Utc::now();
                }

                if self.get_circuit_breaker_state() == "HALF" && (self.get_success_count() - self.get_fail_count() < self.trigger_open_state) {
                    println!("Circuit Breaker: Wechsel Circuit Breaker Status von HALF auf OPEN");
                    self.set_circuit_breaker_state("OPEN");
                    *(self.timestamp.lock().unwrap()) = Utc::now();
                }

                println!("Circuit Breaker: Request ist fehlgeschlagen. Fail Count ist jetzt bei {}", self.get_fail_count());
                return Err(anyhow!("CircuitBreaker: Request failed, return code was 500"))
            },
        }

    }

    // Mutex Variablen: Jeweils der Zugriff als auch das Schreiben wird mit Mutex Sperre belegt
    // Nach dem Block wird jeweils immer die Variable freigegeben
    // Deshalb auch hier die Read operationen in dem Block
    // Wenn sich während der Prüfung der Variablen sich etwas ändert ist das nicht schlimm
    // Beim Schreiben muss es aber sicher sein

    fn increment_success_count(&self) {
        let mut counter = self.success_count.lock().unwrap();
        *counter += 1;
        println!("REST API: Success Count ist {}", *counter);
    }

    fn increment_fail_count(&self) {
        let mut counter = self.fail_count.lock().unwrap();
        *counter += 1;
        println!("REST API: Fail Count ist {}", *counter);
    }

    fn increment_request_count(&self) {
        let mut counter = self.request_count.lock().unwrap();
        *counter += 1;
        println!("REST API: Request Count ist {}", *counter);
    }

    fn set_circuit_breaker_state(&self, new_state: &'a str) {
        let mut state = self.circuit_breaker_state.lock().unwrap();
        *state = new_state;
    }

    fn get_request_count(&self) -> i64 {
        let counter = self.request_count.lock().unwrap();
        return *counter;
    }

    fn get_success_count(&self) -> i64 {
        let counter = self.success_count.lock().unwrap();
        return *counter;
    }

    fn get_fail_count(&self) -> i64{
        let counter = self.fail_count.lock().unwrap();
        return *counter;
    }

    fn get_timestamp(&self) -> DateTime<Utc> {
        let timestamp = self.timestamp.lock().unwrap();
        return *timestamp;
    }

    fn get_circuit_breaker_state(&self) -> &str {
        let state = self.circuit_breaker_state.lock().unwrap();
        return *state;
    }



}


