use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use std::collections::BTreeMap;

pub async fn check_auth_user(login_name: &str, auth_token: &str, is_admin: bool, jwt_key: String) -> Result<(), anyhow::Error> {

    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;

    let claims: BTreeMap<String, String> = auth_token.verify_with_key(&key)?;
    println!("{:?}", claims);

    Ok(())
}
