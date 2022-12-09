use hmac::{Hmac, Mac};
use jwt::{VerifyWithKey};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use chrono::{Utc};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
struct jwt_claims {
    login_name: String,
    is_admin: bool,
    iat: i64
}

#[wasm_bindgen]
pub fn jwt_sign(login_name: String, auth_token: String, is_admin: bool, private_key: String) -> bool {
    // Optional auch match möglich um Fehler genau abzufangen
    let key: Hmac<Sha256> = Hmac::new_from_slice(private_key.as_bytes()).unwrap();
    let claims: jwt_claims = auth_token.verify_with_key(&key).unwrap();
    let jwt_is_admin = claims.is_admin;
    // remove millisecondes
    let formatted_iat = claims.iat / 1000;
    if claims.login_name == login_name.to_string() &&
        ((is_admin && jwt_is_admin) || !is_admin) {
        let current_timestamp = Utc::now();
        let time_diff = current_timestamp.timestamp() - formatted_iat;

        println!("AUTH: timediff ist: {}", time_diff);

        if time_diff > 2000 {
            println!("AUTH: Auth Token ist zu alt");
            return false;
        } else {
            return true;
        }
    } else {
        return false;
    }
}