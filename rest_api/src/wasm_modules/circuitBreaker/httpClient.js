const http = require("http");
class HttpClient {

    async makeRequest(hostname, port, path, login_name, auth_token, method) {

        return new Promise((resolve,reject) => {


            const options = {
                hostname: hostname,
                port: port,
                path: path,
                method: method,
                headers: {
                    "login_name": login_name,
                    "auth_token": auth_token
                },
                timeout: 3000
            };

            let bodyData = {};

            const postData = JSON.stringify(bodyData);

            const req = http.request({
                ...options,
            }, res => {
                const chunks = [];
                res.on('data', data => chunks.push(data))
                res.on('end', () => {
                    let resBody = Buffer.concat(chunks);

                    if(res.statusCode != 200 ) {
                        console.log("Circuit Breaker: HTTP Status Code ist " +  res.statusCode);
                        resolve(res.statusCode);
                    }

                    switch(res.headers['content-type']) {
                        case 'application/json; charset=utf-8':
                            console.log("HTTP Client: Parse JSON Response");
                            resBody = JSON.parse(resBody);
                            break;
                    }
                    console.log("HTTP Client: Post Request war erfolgreich!");
                    resolve(resBody);
                })
            })

            req.on('error',reject);
            if(postData) {
                req.write(postData);
            }
            req.end();
        })
    }
}
module.exports = HttpClient