use wasmedge_http_req::{request::Request, request::Method, uri::Uri};
// wird gebraucht um Namensauflösung zu machen !
use wasmedge_wasi_socket::nslookup;
use anyhow::anyhow;

pub async fn make_request(host: &str, port: i32, path: String, login_name: String, auth_token: String, http_method: String) -> Result<(wasmedge_http_req::response::Response, String), anyhow::Error> {

        let method;
        if http_method == "GET".to_string() {
           method = Method::GET;
        } else if http_method == "POST".to_string() {
                method = Method::POST;
        } else {
                return Err(anyhow!("HTTP Client Method: {} not allowed", http_method));
        }

        let addrs = nslookup(&host, "http")?;
        let addr = format!("{}",addrs[0]);
        let converted_addr = &addr[..(addr.len() - 2)];
        let url = format!("http://{}{}{}", converted_addr, port, path);
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