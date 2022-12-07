use wasmedge_http_req::request;
// wird gebraucht um Namensauflösung zu machen !
use wasmedge_wasi_socket::nslookup;

pub async fn make_post_request(host: &str, port: i32, path: String) -> Result<(wasmedge_http_req::response::Response, String), anyhow::Error> {
        let addrs = nslookup(&host, "http")?;
        let addr = format!("{}",addrs[0]);
        let converted_addr = &addr[..(addr.len() - 2)];
        let mut writer = Vec::new(); //container for body of a response
        const BODY: &[u8; 2] = b"{}";
        // By Default fügt wasmedge hier port 80 ein
        // warum auch immer
        let url = format!("http://{}{}{}", converted_addr, port, path);
        println!("URL IS: {}", url);
        let res = request::post(url, BODY, &mut writer)?;
        // let res = request::get("http://127.0.0.1/", &mut writer).unwrap();
        let response_json = String::from_utf8_lossy(&writer);
        Ok((res, response_json.to_string()))

}