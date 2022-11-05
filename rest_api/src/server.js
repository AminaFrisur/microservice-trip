'use strict';
// TODO: GENERELL -> Authentifizierung zwischen Microservices muss noch umgesetzt werden
// TODO: Umgebungsvariablen beim Start des Containers mit einfügen -> Umgebungsvariable für Router MongoDB
// TODO: Eventuell sollte die Fuhrparkverwaltung die Kommandos von Fahrzeugen übernehmen -> generell mal überlegen wie das genau gemacht werden soll -> ansonsten einfach weglassen


const express = require('express');
const bodyParser = require('body-parser');
var jsonBodyParser = bodyParser.json({ type: 'application/json' });


// Erstelle einen Cache um Token zwischenzuspeichern
var UserCache = require('./cache/userCache.js');
var BookingCache = require('./cache/bookingCache.js');

var userCacheInstance = new UserCache(10000, 10000);
var bookingCacheInstance = new BookingCache(10000, 10000);

let auth = require('./auth.js')();

const PORT = 8000;
const HOST = '0.0.0.0';

var CircuitBreaker = require('./circuitBreaker.js');
var circuitBreakerBenutzerverwaltung = new CircuitBreaker(150, 30, 0, -3,
                                                10, 3,"rest-api-benutzerverwaltung1", "8000", );

var circuitBreakerBuchungsverwaltung = new CircuitBreaker(150, 30, 0, -3,
                                            10, 3,"rest-api-buchungsverwaltung1", "8000", );

const middlerwareWrapperLogin = (cache, isAdmin, circuitBreaker) => {
    return (req, res, next) => {
        auth.checkAuth(req, res, isAdmin, cache, circuitBreaker, next);
    }
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
app.get('/getAllRunningTrips', [middlerwareWrapperLogin(userCacheInstance, true, circuitBreakerBenutzerverwaltung)], async function (req, res) {
    try {
        let cacheEntrys = bookingCacheInstance.getAllCacheEntrys();
        res.status(200).send(cacheEntrys);
    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }
});

// Wird vom Fahrzeug aufgerufen
// Soll im Cache die aktuelle Postion speichern
// In einem Bestimmten Abstand wird dieser Call vom Fahrzeug selbständig aufgerufen
app.post('/updateVehicleLocation', [middlerwareWrapperLogin(userCacheInstance, false, circuitBreakerBenutzerverwaltung), jsonBodyParser], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "longitude", "langtitude", "login_name", "auth_token"]);
        console.log("PARAMETER SIND: ");
        console.log(params);
        let currentBooking = await bookingCacheInstance.checkAndGetBookingInCache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung)
        if(currentBooking && currentBooking.booking && currentBooking.booking.status == "started") {
            currentBooking.booking.longitude = params.longitude;
            currentBooking.booking.langtitude = params.langtitude;
            bookingCacheInstance.updateOrInsertcachedEntrie(currentBooking.index, currentBooking.booking);
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
app.post('/sendVehicleCommand', [middlerwareWrapperLogin(userCacheInstance, false, circuitBreakerBenutzerverwaltung), jsonBodyParser], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "auth_token", "login_name", "Kommando"]);
        let currentBooking = await bookingCacheInstance.checkAndGetBookingInCache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung)


        if(currentBooking && currentBooking.booking && currentBooking.booking.status == "started") {
            bookingCacheInstance.updateOrInsertcachedEntrie(currentBooking.index, currentBooking.booking);
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
app.post('/startTrip/:buchungsNummer', [middlerwareWrapperLogin(userCacheInstance, false, circuitBreakerBenutzerverwaltung)], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "login_name", "auth_token"]);
        let currentBooking = await bookingCacheInstance.checkAndGetBookingInCache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung)
        console.log(currentBooking);
        if(currentBooking && currentBooking.booking && currentBooking.booking.status == "paid") {
            // let headerData = { 'Content-Type': 'application/json', 'auth_token': params.auth_token, 'login_name': params.login_name};

            let response = await circuitBreakerBuchungsverwaltung.circuitBreakerRequest("/startTrip/" + params.buchungsNummer, {}, {}, "POST");
            if(response) {
                currentBooking.booking.status = "started";
                bookingCacheInstance.updateOrInsertcachedEntrie(currentBooking.index, currentBooking.booking);
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
app.post('/endTrip/:buchungsNummer', [middlerwareWrapperLogin(userCacheInstance, false, circuitBreakerBenutzerverwaltung)], async function (req, res) {
    try {
        let params = checkParams(req, res,["buchungsNummer", "login_name", "auth_token"]);
        let currentBooking = await bookingCacheInstance.checkAndGetBookingInCache(params.login_name, params.auth_token, params.buchungsNummer, circuitBreakerBuchungsverwaltung);
        if(currentBooking && currentBooking.booking && currentBooking.booking.status == "started") {

            // let headerData = { 'Content-Type': 'application/json', 'auth_token': params.auth_token, 'login_name': params.login_name};

            await circuitBreakerBuchungsverwaltung.circuitBreakerRequest("/endTrip/" + params.buchungsNummer, {}, {}, "POST");
            bookingCacheInstance.removeFromCache(currentBooking.index);
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
