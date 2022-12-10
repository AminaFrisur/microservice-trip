'use strict';

const express = require('express');
const bodyParser = require('body-parser');
var jsonBodyParser = bodyParser.json({ type: 'application/json' });

var BookingCache = require('./wasm_modules/bookingCache/pkg/bookingCache.js');
var bookingCacheInstance = new BookingCache.Cache(BigInt(10000), BigInt(10000));

var CircuitBreaker = require('./wasm_modules/circuitBreaker/pkg/CircuitBrekaer.js');
var circuitBreakerBuchungsverwaltung = new CircuitBreaker.CircuitBreaker(BigInt(150), BigInt(30), BigInt(0), BigInt(-3),
    BigInt(10), BigInt(3), process.env.BUCHUNGSVERWALTUNG, 80);

var HttpClient = require('./httpClient.js');
var httpClient = new HttpClient();

let Auth = require('./auth.js')();

const PORT = 8000;
const HOST = '0.0.0.0';
const JWT_SECRET = "goK!pusp6ThEdURUtRenOwUhAsWUCLheasfr43qrf43rttq3";


function parseBookingWasmPointer(bookingPointer) {
    let booking = new BookingCache.Booking(
        bookingPointer.get_buchungsNummer(), bookingPointer.get_buchungsDatum(),
        bookingPointer.get_loginName(), bookingPointer.get_dauerDerBuchung(),
        bookingPointer.get_fahrzeugId(), bookingPointer.get_preisNetto(),
        bookingPointer.get_status(), bookingPointer.get_longitude(),
        bookingPointer.get_langtitude()
    )

    return booking;
}

const middlerwareCheckAuth = (isAdmin) => {
    return (req, res, next) => {
        Auth.checkAuth(req, res, isAdmin, JWT_SECRET,  next);
    }
}

var rootBooking = "root";
var passwordBooking = process.env.BOOKINGPW;


function checkParams(req, res, requiredParams) {
    console.log("checkParams", requiredParams);
    let paramsToReturn = {};
    for (let i = 0; i < requiredParams.length; i++) {
            let param = requiredParams[i];
            
        if (!(req.headers && param in req.headers)
            && !(req.query && param in req.query)
            && !(req.body && param in req.body)
            && !(req.params && param in req.params)) {
            let error = "error parameter " + param + " is missing";
            console.log(error);
            throw error;
            return;
        }

        if (req.headers && param in req.headers) {
            paramsToReturn[param] = req.headers[param];
        }

        if (req.query && param in req.query) {
            paramsToReturn[param] = req.query[param];
        }
        if (req.body && param in req.body) {
            paramsToReturn[param] = req.body[param];
        }
        if (req.params && param in req.params) {
            paramsToReturn[param] = req.params[param];
        }
    }
    return  paramsToReturn;
}

// App
const app = express();

// nur für Admin
app.get('/getAllRunningTrips', [middlerwareCheckAuth(true)], async function (req, res) {
    try {
        // let cacheEntrys = bookingCacheInstance.getAllCacheEntrys();
        res.status(500).send("AKTUELL NICHT SUPPORTED");
    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }
});

// Wird vom Fahrzeug aufgerufen
// Soll im Cache die aktuelle Postion speichern
// In einem Bestimmten Abstand wird dieser Call vom Fahrzeug selbständig aufgerufen
app.post('/sendVehicleCommand', [middlerwareCheckAuth(false), jsonBodyParser], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "auth_token", "login_name", "Kommando"]);
        let result = await bookingCacheInstance.check_and_get_booking_in_cache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung, httpClient);
        let currentBooking = result.get_booking();
        let bookingFound =  result.get_bookingFound();
        if(currentBooking.get_status() == "started") {

            let booking = parseBookingWasmPointer(currentBooking);
            bookingCacheInstance.update_or_insert_cached_entrie(bookingFound, result.get_index(), booking);
            // TODO: Mockup Request zu Fahrzeug
            res.status(200).send("Fahrzeug Kommando ausgeführt");
        } else {
            throw "Buchung konnte unter angegebener Buchungsnummer und Nutzername nicht gefunden werden !"
        }

    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }
});

app.post('/updateVehicleLocation', [middlerwareCheckAuth(false), jsonBodyParser], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "longitude", "langtitude", "login_name", "auth_token"]);
        let result = await bookingCacheInstance.check_and_get_booking_in_cache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung, httpClient);
        let currentBooking = result.get_booking();
        let bookingFound =  result.get_bookingFound();
        if(currentBooking.get_buchungsNummer() > 0) {
            currentBooking.set_longitude(BigInt(params.longitude));
            currentBooking.set_langtitude(BigInt(params.langtitude));
            let booking = parseBookingWasmPointer(currentBooking);
            bookingCacheInstance.update_or_insert_cached_entrie(bookingFound, result.get_index(), booking);
            res.status(200).send("Fahrzeug Standort wurde aktualisiert");
        } else {
            throw "Buchung konnte unter angegebener Buchungsnummer und Nutzername nicht gefunden werden !"
        }

    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }
});
// Starte den Trip
// Rufe dazu die Buchungsverwaltung auf
// Wenn Start Trip aufgerufen wird, sollte die Buchung in keinem Fall im Cache liegen !
app.post('/startTrip/:buchungsNummer', [middlerwareCheckAuth(false)], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "login_name", "auth_token"]);
        let result = await bookingCacheInstance.check_and_get_booking_in_cache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung, httpClient);
        let currentBooking = result.get_booking();
        let bookingFound =  result.get_bookingFound();
        if(currentBooking.get_buchungsNummer() > 0 && currentBooking.get_status() == "paid") {
            let response = await circuitBreakerBuchungsverwaltung.circuit_breaker_post_request("/startTrip/" + params.buchungsNummer, params.login_name, params.auth_token, "POST", httpClient);
            console.log("RESPONSE IST:");
            console.log(response);
            let http_code = parseInt(response, 10);
            console.log("http_code IST:");
            console.log(http_code);
            if(Number.isInteger(http_code)) {
                res.status(http_code).send("Start Trip: Buchung konnte nicht gestartet werden !");
            } else {
                currentBooking.set_status("started");
                let booking = parseBookingWasmPointer(currentBooking);
                bookingCacheInstance.update_or_insert_cached_entrie(bookingFound, result.get_index(), booking);
                res.status(200).send("Trip wurde gestartet");
            }
        } else {
            throw "Start Trip: Buchung konnte nicht unter angegebener Buchungsnummer, Nutzername und dem Status paid gefunden werden !"
        }

    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }

});

// Starte den Trip
// Rufe dazu die Buchungsverwaltung auf
app.post('/endTrip/:buchungsNummer', [middlerwareCheckAuth(false)], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "login_name", "auth_token"]);
        let result = await bookingCacheInstance.check_and_get_booking_in_cache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung, httpClient);
        let currentBooking = result.get_booking();
        let bookingFound =  result.get_bookingFound();
        if(currentBooking.get_buchungsNummer() > 0 && currentBooking.get_status() == "started") {
            let response = await circuitBreakerBuchungsverwaltung.circuit_breaker_post_request("/endTrip/" + params.buchungsNummer, params.login_name, params.auth_token, "POST", httpClient);
            console.log("RESPONSE IST:");
            console.log(response);
            let http_code = parseInt(response, 10);
            console.log("http_code IST:");
            console.log(http_code);
            if(Number.isInteger(http_code)) {
                res.status(http_code).send("End Trip: Buchung konnte nicht beendet werden !");
            } else {
                if(bookingFound) {
                    bookingCacheInstance.remove_from_cache(result.get_index());
                }
            }
        } else {
            throw "End Trip: Buchung konnte nicht unter angegebener Buchungsnummer, Nutzername und dem Status started gefunden werden !"
        }
        res.status(200).send("Trip wurde beendet");

    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }

});

app.listen(PORT, HOST, () => {
    console.log(`Running on http://${HOST}:${PORT}`);
});
