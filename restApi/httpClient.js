import {fetch} from "http";
export async function  makeRequest(hostname, port, path, bodyData, headerData, method) {

        let parsedBodyData = null
        if(method === "POST") {
            parsedBodyData = JSON.stringify(bodyData);
        }
        let resp = await fetch("http://" + hostname + ":" + port + path, { method: method, body: parsedBodyData, headers: headerData });
        if(resp.status != 200) {
                return false;
        }
        const response = await resp.text();
        console.log("test");
        console.log(response);
        return response;
}

