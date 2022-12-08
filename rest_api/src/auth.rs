use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, Token, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct jwt_claims {
    login_name: String,
    is_admin: String,
    iat: i64
}




pub async fn check_auth_user(login_name: &str, auth_token: &str, is_admin: bool, jwt_key: String) -> Result<(), anyhow::Error> {


    println!("CHECK JWT");
    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;

    let map: jwt_claims = auth_token.verify_with_key(&key).unwrap();
    // let token: Token<Header, BTreeMap<String, <T>, _> = VerifyWithKey::verify_with_key(auth_token, &key).unwrap();


    println!("CHECK JWT ENDING");
    // println!("{:?}", map::<BTreeMap<std::string::String, _>>);

    Ok(())
}
