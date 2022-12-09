use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, Token, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use std::time::Instant;
use anyhow::anyhow;

#[derive(Serialize, Deserialize)]
struct jwt_claims {
    login_name: String,
    is_admin: bool,
    iat: i64
}

pub async fn check_auth_user(login_name: &str, auth_token: &str, is_admin: bool, jwt_key: String) -> Result<(), anyhow::Error> {

    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;
    let claims: jwt_claims = auth_token.verify_with_key(&key)?;
    // let jwt_is_admin: bool = claims.is_admin.parse()?;
    let jwt_is_admin= claims.is_admin;
    // remove millisecondes
    let formatted_iat = claims.iat / 1000;

    if claims.login_name == login_name.to_string() &&
        ((is_admin && jwt_is_admin) || !is_admin ){

        let current_timestamp = Utc::now();
        let time_diff = current_timestamp.timestamp() -  formatted_iat;

        println!("AUTH: timediff ist: {}", time_diff);

        if time_diff > 2000 {
            println!("AUTH: Auth Token ist zu alt");
            return Err(anyhow!("token is to old. Please get a new one."));
        } else {
            return Ok(());
        }

    } else {
        return Err(anyhow!("token and/or login name are missing or are not valid"));
    }


}
