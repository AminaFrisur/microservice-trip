use http_req::{request::Request, request::Method, uri::Uri};
// wird gebraucht um Namensauflösung zu machen !
use anyhow::anyhow;
use std::net::{SocketAddr, ToSocketAddrs};
pub async fn make_request(host: &str, port: i32, path: String, login_name: String, auth_token: String, http_method: String) -> Result<(http_req::response::Response, String), anyhow::Error> {

        let method;
        if http_method == "GET".to_string() {
           method = Method::GET;
        } else if http_method == "POST".to_string() {
                method = Method::POST;
        } else {
                return Err(anyhow!("HTTP Client Method: {} not allowed", http_method));
        }

        // let addrs = host.to_socket_addrs().unwrap();;
        // let addr = format!("{}", Some(addrs));
        // let converted_addr = &addr[..(addr.len() - 2)];
        let url = format!("http://{}{}{}", "0.0.0.0", port, path);
        println!("URL IST: {}", url);

        let converted_addr: Uri = Uri::try_from(&url[..])?;
        // let converted_addr = &addr[..(addr.len() - 2)];
        let mut writer = Vec::new(); //container for body of a response
        const body: &[u8; 2] = b"{}";
        let res = Request::new(&converted_addr)
            .method(method)
            .header("login_name", &login_name)
            .header("auth_token", &auth_token)
            .body(body)
            .send(&mut writer)?;

        let response_json = String::from_utf8_lossy(&writer);
        Ok((res, response_json.to_string()))

}