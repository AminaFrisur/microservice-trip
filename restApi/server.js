import http from 'http';
// import url from 'url';
import CircuitBreaker from './circuitBreaker.js';
import BookingCache from "./cache/bookingCache.js";
import UserCache from "./cache/userCache.js"
import {checkAuth} from "./auth.js"

async function parseJsonBody(req) {

    return new Promise((resolve,reject) => {
        var json = '';
        try {
            req.on('data', function (chunk) {
                json += chunk.toString('utf8');
            });
            req.on('end', function () {
                req["jsonBody"] = JSON.parse(json)
                resolve()
            });
        } catch (e) {
            reject(e);
        }
    });
}

// Checkparams Funktion
function checkParams(req, requiredParams) {
    let paramsToReturn = {};
    for (let i = 0; i < requiredParams.length; i++) {
        let param = requiredParams[i];

        if (!(req.headers && param in req.headers)
            &&!(req.jsonBody && param in req.jsonBody))
        {
            let error = "error parameter " + param + " is missing";
            throw error;
            return;
        }

        if (req.headers && param in req.headers) {
            paramsToReturn[param] = req.headers[param];
        }

        if (req.jsonBody && param in req.jsonBody) {
            paramsToReturn[param] = req.jsonBody[param];
        }
    }
    return paramsToReturn;
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

    if (req.url === "/getAllRunningTrips" && req.method.toUpperCase() === "GET") {
        try {
            let params = checkParams(req, ["auth_token", "login_name"]);
            await checkAuth(true, params.login_name, params.auth_token, userCacheInstance, circuitBreakerBenutzerverwaltung);
            let cacheEntrys = bookingCacheInstance.getAllCacheEntrys();
            res.writeHead(200, { "Content-Type": "application/json" });
            res.write(JSON.stringify(cacheEntrys));
            res.end();
        } catch(err) {
            res.writeHead(401, { "Content-Type": "text/plain" });
            res.end(err);
        }
    } else if(req.url === "/updateVehicleLocation" && req.method.toUpperCase() === "POST") {
        try {

            await parseJsonBody(req);
            let params = checkParams(req, ["buchungsNummer", "longitude", "langtitude", "auth_token", "login_name"]);
            await checkAuth(true, params.login_name, params.auth_token, userCacheInstance, circuitBreakerBenutzerverwaltung);
            let currentBooking = await bookingCacheInstance.checkAndGetBookingInCache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung);
            console.log(currentBooking.booking.status);
            if(currentBooking && currentBooking.booking && currentBooking.booking.status == "started") {
                currentBooking.booking.longitude = params.longitude;
                currentBooking.booking.langtitude = params.langtitude;
                bookingCacheInstance.updateOrInsertcachedEntrie(currentBooking.index, currentBooking.booking);
                res.writeHead(200, { "Content-Type": "text/plain" });
                res.write("Fahrzeug Standort wurde aktualisiert");
                res.end();
            } else {
                res.writeHead(401, { "Content-Type": "text/plain" });
                res.write("Buchung konnte unter angegebener Buchungsnummer und Nutzername nicht gefunden werden !");
                res.end();
            }

        } catch(err){
            console.log(err);
            res.writeHead(401, { "Content-Type": "text/plain" });
            res.write("Microservice interner Fehler");
            res.end();
        }

    } else if(req.url === "/sendVehicleCommand" && req.method.toUpperCase() === "POST") {
        try {
            await parseJsonBody(req);
            let params = checkParams(req, ["buchungsNummer", "auth_token", "login_name", "kommando"]);
            await checkAuth(true, params.login_name, params.auth_token, userCacheInstance, circuitBreakerBenutzerverwaltung);
            let currentBooking = await bookingCacheInstance.checkAndGetBookingInCache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung);
            console.log(currentBooking);
            if(currentBooking && currentBooking.booking && currentBooking.booking.status == "started") {
                bookingCacheInstance.updateOrInsertcachedEntrie(currentBooking.index, currentBooking.booking);
                // TODO: Mockup Request zu Fahrzeug
                res.writeHead(200, {"Content-Type": "text/plain"});
                res.write("Fahrzeug Kommando ausgeführt");
                res.end();
            } else {
                throw "Buchung konnte unter angegebener Buchungsnummer und Nutzername nicht gefunden werden !"
            }
        } catch(err) {
            res.writeHead(401, { "Content-Type": "text/plain" });
            res.end(err);
        }

    } else if(req.url.match(/\/startTrip\/([0-9]+)/) && req.method.toUpperCase() === "POST") {
        try {
            const buchungsNummer = req.url.split("/")[2];
            console.log(buchungsNummer);
            let headerParams = checkParams(req, ["auth_token", "login_name"]);
            await checkAuth(false, headerParams.login_name, headerParams.auth_token, userCacheInstance, circuitBreakerBenutzerverwaltung);
            let currentBooking = await bookingCacheInstance.checkAndGetBookingInCache(headerParams.login_name, headerParams.auth_token, buchungsNummer, circuitBreakerBuchungsverwaltung);
            if(currentBooking && currentBooking.booking && currentBooking.booking.status == "paid") {
                // let headerData = { 'Content-Type': 'application/json', 'auth_token': params.auth_token, 'login_name': params.login_name};

                let response = await circuitBreakerBuchungsverwaltung.circuitBreakerRequest("/startTrip/" + buchungsNummer, {}, {}, "POST");
                if(response) {
                    currentBooking.booking.status = "started";
                    bookingCacheInstance.updateOrInsertcachedEntrie(currentBooking.index, currentBooking.booking);
                    res.writeHead(200, { "Content-Type": "text/plain" });
                    res.write("Trip wurde gestartet");
                    res.end();
                } else {
                    res.writeHead(401, { "Content-Type": "text/plain" });
                    res.end("Buchung konnte nicht gestartet werden !");
                }

            } else {
                res.writeHead(401, { "Content-Type": "text/plain" });
                res.end("Buchung konnte nicht unter angegebener Buchungsnummer, Nutzername und dem Status started gefunden werden !");
            }
        } catch (err) {
            console.log(err);
            res.writeHead(401, { "Content-Type": "text/plain" });
            res.end("Interner Fehler im Microservice aufgetreten");
        }

    } else if(req.url.match(/\/endTrip\/([0-9]+)/) && req.method.toUpperCase() === "POST") {
        try {
            const buchungsNummer = req.url.split("/")[2];
            let headerParams = checkParams(req, ["auth_token", "login_name"]);
            await checkAuth(false, headerParams.login_name, headerParams.auth_token, userCacheInstance, circuitBreakerBenutzerverwaltung);
            let currentBooking = await bookingCacheInstance.checkAndGetBookingInCache(headerParams.login_name, headerParams.auth_token, buchungsNummer, circuitBreakerBuchungsverwaltung);
            if(currentBooking && currentBooking.booking && currentBooking.booking.status == "started") {

                // let headerData = { 'Content-Type': 'application/json', 'auth_token': params.auth_token, 'login_name': params.login_name};

                await circuitBreakerBuchungsverwaltung.circuitBreakerRequest("/endTrip/" + buchungsNummer, {}, {}, "POST");
                bookingCacheInstance.removeFromCache(currentBooking.index);
                res.writeHead(200, { "Content-Type": "text/plain" });
                res.end("Trip wurde beendet");
            } else {
                res.writeHead(401, { "Content-Type": "text/plain" });
                res.end("Buchung konnte nicht unter angegebener Buchungsnummer, Nutzername und dem Status started gefunden werden !");
            }
        } catch(err) {
            console.log(err);
            res.writeHead(401, { "Content-Type": "text/plain" });
            res.end("Interner Fehler im Microservice aufgetreten");
        }
    }

    else {
        res.writeHead(404, { "Content-Type": "text/plain" });
        res.end("Route not found" );
    }
});

server.listen(port, hostname, () => {
    console.log(`Server running at http://${hostname}:${port}/`);
});

