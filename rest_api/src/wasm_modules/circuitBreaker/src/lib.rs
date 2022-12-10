use chrono::{DateTime, Utc};
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: String);
}

#[wasm_bindgen(raw_module = "../../httpClient.js")]
extern "C" {

    pub type HttpClient;

    #[wasm_bindgen(method, catch)]
    pub async fn makeRequest(this: &HttpClient, hostname: String, port: i32, path: String, login_name: String, auth_token: String, method: String) -> Result<JsValue,JsValue>;
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct CircuitBreaker {
    circuit_breaker_state: String,
    success_count: i64,
    fail_count: i64,
    timeout_reset: i64,
    timeout_open_state: i64,
    trigger_half_state: i64,
    trigger_open_state: i64,
    trigger_closed_state: i64,
    timestamp:  DateTime<Utc>,
    permitted_requests_in_state_half: i64,
    request_count: i64,
    hostname:  String,
    port: i32,
}
#[wasm_bindgen]
impl CircuitBreaker {
    #[wasm_bindgen(constructor)]
    pub fn new(timeout_reset: i64, timeout_open_state: i64, trigger_half_state: i64, trigger_open_state: i64,
               permitted_requests_in_state_half: i64, trigger_closed_state: i64, hostname: String, port: i32) -> Self {

        return Self { circuit_breaker_state: "CLOSED".to_string(), success_count: 0, fail_count: 0, timeout_reset, timeout_open_state,
            trigger_half_state, trigger_closed_state, timestamp: Utc::now(), trigger_open_state, permitted_requests_in_state_half,
            request_count: 0, hostname, port};
    }

    fn check_reset(&mut self, time_diff: i64) {
        log(format!("{}", time_diff));
        log(format!("{}", time_diff - self.timeout_reset));
        log("Circuit Breaker: Prüfe ob Circuit Breaker Status zurückgesetzt werden soll".to_string());
        if time_diff > self.timeout_reset &&
            (self.circuit_breaker_state == "CLOSED" || self.circuit_breaker_state == "HALF") {
            log("Circuit Breaker: Kompletter Status wird zurückgesetzt!".to_string());
            self.fail_count = 0;
            self.success_count = 0;
            self.timestamp = Utc::now();
            self.request_count = 0;
        }
    }

    #[wasm_bindgen(method)]
    pub async fn circuit_breaker_post_request(&mut self, addr_with_params: String, login_name: String, auth_token: String, http_method: String, client: HttpClient) -> String {

        log(format!("REST API: AKTUELLER CIRCUIT BREAKER STATUS IST: {}", self.circuit_breaker_state));

        let current_timestamp = Utc::now();
        let time_diff = current_timestamp.signed_duration_since(self.timestamp).num_seconds();
        log(format!("timeDiff is {}", time_diff));
        log(format!("timeout_open_state ist {}", self.timeout_open_state));
        self.check_reset(time_diff);

        if self.circuit_breaker_state == "OPEN".to_string() {

            if time_diff >= self.timeout_open_state {
                // Wenn timeout abgelaufen setze den Circuit Breaker wieder auf HALF
                log("Circuit Breaker: Wechsel Circuit Breaker Status von OPEN auf HALF".to_string());
                self.circuit_breaker_state = "HALF".to_string();

            } else {
                log("Circuit Breaker: immer noch auf Zustand OPEN".to_string());
                log(format!("Circuit Breaker ist {} noch offen", (self.timeout_open_state - time_diff)));
                log("Request fehlgeschlagen: Circuit Breaker ist im Zustand offen und keine Requests sind zum Service Benutzerverwaltung erlaubt".to_string());
                return "500".to_string();
            }

        }

        if self.circuit_breaker_state == "HALF".to_string() && self.request_count > self.permitted_requests_in_state_half {
            log("Request fehlgeschlagen: Circuit Breaker ist auf Zustand HALF aber der erlaubte RequestCount ist erreicht".to_string());
            return "500".to_string();
        }

        if self.circuit_breaker_state == "HALF".to_string() && (self.success_count - self.fail_count > self.trigger_closed_state) {
            self.check_reset(time_diff);
            log("Circuit Breaker: Wechsel Circuit Breaker Status von HALF auf CLOSED".to_string());
            self.circuit_breaker_state = "CLOSED".to_string();
        }

        log("Circuit Breaker: Führe HTTP Request im Circuit Breaker durch".to_string());
        if self.circuit_breaker_state == "HALF".to_string() {
            self.request_count += 1;
        }

        match client.makeRequest( self.hostname.clone(), self.port, addr_with_params, login_name, auth_token, http_method).await {
            Ok(res) => {
                self.success_count += 1;
                log(format!("Circuit Breaker: Request war erfolgreich. Success Count ist jetzt bei {}", self.success_count));
                log(format!("Ciruit Breaker Response ist {:?}", res));
                // Response ist String
                // Somit ist Response HTTP Return Status = 200
                match res.as_string() {
                    Some(s) => {return format!("{}",s )},
                    None => { log("Circuit Breaker: Request war erfolgreich, aber response ist nicht 20ß".to_string());},
                }

                // Request ist Erfolgreich
                // HTTP CLient gibt aber HTTP STatus code zurück
                // Ist dann != 200
                let response_json = res.as_f64();
                match response_json {
                    Some(s) => {return format!("{}",s )},
                    // Wenn es weder String noch f64 ist -> Status Code 500
                    // Dann ist etwas komplett schief gegangen
                    None => {return "500".to_string()},
                }
            },
            Err(err) => {
                log(format!("{:?}", err));

                self.fail_count += 1;
                if self.circuit_breaker_state == "CLOSED".to_string() && self.success_count - self.fail_count < self.trigger_half_state {
                    log("Circuit Breaker: Wechsel Circuit Breaker Status von CLOSED auf HALF".to_string());
                    self.circuit_breaker_state = "HALF".to_string();
                    self.timestamp = Utc::now();
                }

                if self.circuit_breaker_state == "HALF".to_string() && (self.success_count - self.fail_count < self.trigger_open_state) {
                    log("Circuit Breaker: Wechsel Circuit Breaker Status von HALF auf OPEN".to_string());
                    self.circuit_breaker_state = "OPEN".to_string();
                    self.timestamp = Utc::now();
                }

                log(format!("Circuit Breaker: Request ist fehlgeschlagen. Fail Count ist jetzt bei {}", self.fail_count));
                return "500".to_string();
            },
        }

    }


}