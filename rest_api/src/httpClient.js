const http = require("http");
module.exports = function() {
    var module = {};

    // Gibt entweder ein richtiges Ergebnis zurück
    // oder Boolean False falls http Code nicht 200
    // oder schmeißt eine Exception, falls Timeout beispielsweise erreicht
    module.makePostRequest = async function(hostname, port, path, bodyData, headerData) {

        // TODO: TIMEOUT FUNKTIONIERT AUS IRGEND EINEM GRUND NICHT
        return new Promise((resolve,reject) => {

            const options = {
                hostname: hostname,
                port: port,
                path: path,
                method: 'POST',
                headers: {
                    ...headerData
                },
                timeout: 3000
            };

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
                        resolve(false);
                    }

                    switch(res.headers['content-type']) {
                        // TODO: Was tun wenn der reponse text ist ?
                        case 'application/json; charset=utf-8':
                            console.log("Circuit Breaker: Parse JSON Response");
                            resBody = JSON.parse(resBody);
                            break;
                    }
                    console.log("Circuit Breaker: Post Request war erfolgreich!");
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

    return module;
}
