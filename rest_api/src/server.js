'use strict';

const express = require('express');
const bodyParser = require('body-parser');
var jsonBodyParser = bodyParser.json({ type: 'application/json' });

var BookingCache = require('./wasm_modules/bookingCache/pkg/bookingCache.js');
var bookingCacheInstance = new BookingCache.Cache(BigInt(10000), BigInt( 10000));

let Auth = require('./auth.js')();

const PORT = 8000;
const HOST = '0.0.0.0';

var CircuitBreaker = require('./circuitBreaker.js');
var circuitBreakerBuchungsverwaltung = new CircuitBreaker(150, 30, 0, -3,
                                            10, 3,process.env.BUCHUNGSVERWALTUNG, "80");

const JWT_SECRET = "goK!pusp6ThEdURUtRenOwUhAsWUCLheasfr43qrf43rttq3";

const middlerwareCheckAuth = (isAdmin) => {
    return (req, res, next) => {
        Auth.checkAuth(req, res, isAdmin, JWT_SECRET,  next);
    }
}

var rootBooking = "root";
var passwordBooking = process.env.BOOKINGPW;


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
app.post('/updateVehicleLocation', [middlerwareCheckAuth(false), jsonBodyParser], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "longitude", "langtitude", "login_name", "auth_token"]);
        let result = await bookingCacheInstance.check_and_get_booking_in_cache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung);
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

// TODO: Schauen ob das überhaupt implementiert werden soll
app.post('/sendVehicleCommand', [middlerwareCheckAuth(false), jsonBodyParser], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "auth_token", "login_name", "Kommando"]);
        let result = await bookingCacheInstance.check_and_get_booking_in_cache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung);
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

// Starte den Trip
// Rufe dazu die Buchungsverwaltung auf
// Wenn Start Trip aufgerufen wird, sollte die Buchung in keinem Fall im Cache liegen !
app.post('/startTrip/:buchungsNummer', [middlerwareCheckAuth(false)], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "login_name", "auth_token"]);
        let result = await bookingCacheInstance.check_and_get_booking_in_cache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung);
        let currentBooking = result.get_booking();
        let bookingFound =  result.get_bookingFound();
        if(currentBooking.get_status() == "paid") {
            let response = await circuitBreakerBuchungsverwaltung.circuitBreakerRequest("/startTrip/" + params.buchungsNummer, params.login_name, params.auth_token, "POST");
            if(response) {
                currentBooking.set_status("started");
                let booking = parseBookingWasmPointer(currentBooking);
                bookingCacheInstance.update_or_insert_cached_entrie(bookingFound, result.get_index(), booking);
                res.status(200).send("Trip wurde gestartet");
            } else {
                throw "Buchung konnte nicht gestartet werden !"
            }

        } else {
            throw "Buchung konnte nicht unter angegebener Buchungsnummer, Nutzername und dem Status started gefunden werden !"
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
        let result = await bookingCacheInstance.check_and_get_booking_in_cache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung);
        let currentBooking = result.get_booking();
        let bookingFound =  result.get_bookingFound();
        if(currentBooking.get_status() == "started") {
            await circuitBreakerBuchungsverwaltung.circuitBreakerRequest("/endTrip/" + params.buchungsNummer, params.login_name, params.auth_token, "POST");
            if(bookingFound) {
                bookingCacheInstance.remove_from_cache(result.get_index());
            }

        } else {
            throw "Buchung konnte unter angegebener Buchungsnummer und Nutzername nicht gefunden werden !"
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
