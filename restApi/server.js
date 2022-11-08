import http from 'http';
// import url from 'url';
import CircuitBreaker from './circuitBreaker.js';
import BookingCache from "./cache/bookingCache.js";
import UserCache from "./cache/userCache.js"
import {checkAuth} from "./auth.js"

// Checkparams Funktion
function checkParams(foundParams, requiredParams) {
    console.log("checkParams");
    let paramsToReturn = {};
    for (let i = 0; i < requiredParams.length; i++) {
        let param = requiredParams[i];
        console.log("CHECKPARAM:" + param);

        if (!(foundParams[i])){
            let error = "error parameter " + param + " is missing";
            console.log(error);
            throw error;
            return;
        }
        paramsToReturn[param] = foundParams[i];
    }
    return  paramsToReturn;
}


// Erstelle jeweils einen Cache um Token und Buchung zwischenzuspeichern
var userCacheInstance = new UserCache(10000, 10000);
var bookingCacheInstance = new BookingCache(10000, 10000);

// CircuitBreaker für Benutzerverwaltung
var circuitBreakerBenutzerverwaltung = new CircuitBreaker(150, 30, 0, -3,
    10, 3,"localhost", "8000", );

// CircuitBreaker für die Buchungsverwaltung
var circuitBreakerBuchungsverwaltung = new CircuitBreaker(150, 30, 0, -3,
    10, 3,"localhost", "8002", );

// Konstanten
const port = 8003;
const hostname = '0.0.0.0';

const server = http.createServer(async (req, res) => {

    // authToken aus dem Header holen
    const loginName = req.headers["login_name"];
    const authToken = req.headers["auth_token"];

    if (req.url === "/getAllRunningTrips" && req.method === "Get") {
        try {
            console.log("Start getAllRunningTrips");
            await checkAuth(true, loginName, authToken, userCacheInstance, circuitBreakerBenutzerverwaltung);
            let cacheEntrys = bookingCacheInstance.getAllCacheEntrys();
            res.writeHead(200, { "Content-Type": "application/json" });
            res.write(cacheEntrys);
            res.end();
        } catch(err) {
            res.writeHead(401, { "Content-Type": "application/json" });
            res.end(JSON.stringify({ message: err }));
        }
    } else if(req.url === "/updateVehicleLocation" && req.method === "Post") {
        res.writeHead(200, { "Content-Type": "application/json" });
        res.write("updateVehicleLocation from NodeJS");
        res.end();

    } else if(req.url === "/sendVehicleCommand" && req.method === "Post") {
        res.writeHead(200, { "Content-Type": "application/json" });
        res.write("sendVehicleCommand from NodeJS");
        res.end();

    } else if(req.url.match(/\/startTrip\/([0-9]+)/) && req.method === "Post") {
        try {
            const buchungsNummer = req.url.split("/")[2];
            let params = checkParams([buchungsNummer], ["buchungsNummer"]);
            res.writeHead(200, { "Content-Type": "application/json" });
            res.write("startTrip from NodeJS");
            res.end();
        } catch(err) {
            res.writeHead(400, { "Content-Type": "application/json" });
            res.end(JSON.stringify({ message: err }));
        }

    } else if(req.url === "/endTrip" && req.method === "Post") {
        try {
            const buchungsNummer = req.url.split("/")[2];
            let params = checkParams([buchungsNummer], ["buchungsNummer"]);
            res.writeHead(200, { "Content-Type": "application/json" });
            res.write("endTrip from NodeJS");
            res.end();
        } catch(err) {
            res.writeHead(400, { "Content-Type": "application/json" });
            res.end(JSON.stringify({ message: err }));
        }
    }

    else {
        res.writeHead(404, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ message: "Route not found" }));
    }
});

server.listen(port, hostname, () => {
    console.log(`Server running at http://${hostname}:${port}/`);
});

