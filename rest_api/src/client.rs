use wasmedge_http_req::{request::Request, request::Method, uri::Uri};
// wird gebraucht um Namensauflösung zu machen !
use wasmedge_wasi_socket::nslookup;

pub async fn make_post_request(host: &str, port: i32, path: String, login_name: String, auth_token: String) -> Result<(wasmedge_http_req::response::Response, String), anyhow::Error> {
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
            .method(Method::POST)
            .header("login_name", &login_name)
            .header("auth_token", &auth_token)
            .body(body)
            .send(&mut writer)?;

        let response_json = String::from_utf8_lossy(&writer);
        Ok((res, response_json.to_string()))

}