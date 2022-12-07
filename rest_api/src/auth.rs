use jsonwebtoken::{decode, Validation};

pub async fn check_auth_user(login_name: &str, auth_token: &str, is_admin: bool, jwt_key: String) -> Result<(), anyhow::Error> {

    let result = decode::<Claims>(&token, jwt_key , &Validation::default())?;
    println!("{?}", result.claims);

    Ok(())
}
