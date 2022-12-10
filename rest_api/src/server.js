'use strict';

const express = require('express');
const bodyParser = require('body-parser');
var jsonBodyParser = bodyParser.json({ type: 'application/json' });

let Auth = require('./auth.js')();

const PORT = 8000;
const HOST = '0.0.0.0';

var CircuitBreaker = require('./wasm_modules/bookingCache/circuitBreaker.js');
var bookingCache = require( './wasm_modules/bookingCache/pkg/bookingCache.js');
var bookingCacheInstance = new bookingCache.Cache(BigInt(10000), BigInt(10000));
var circuitBreakerBuchungsverwaltung = new CircuitBreaker(150, 30, 0, -3,
                                            10, 3,process.env.BUCHUNGSVERWALTUNG, "80");
var CircuitBreakerWasm = require('./wasm_modules/circuitBreaker/pkg/CircuitBrekaer.js');

var circuitBreakerBuchungsverwaltungWasm = new CircuitBreakerWasm.CircuitBreaker(BigInt(150), BigInt(30), BigInt(0),
    BigInt(-3), BigInt(10), BigInt(3), process.env.BUCHUNGSVERWALTUNG, 80);

var HttpClient = require("./wasm_modules/circuitBreaker/httpClient.js");
const bookingCacheWithCircuitBreaker = require("./wasm_modules/bookingCacheAndCiruitBreaker/pkg/bookingCache");
const circuitBreaker = require("./wasm_modules/bookingCacheAndCiruitBreaker/CircuitBrekaer");
var client = new HttpClient();

const JWT_SECRET = "goK!pusp6ThEdURUtRenOwUhAsWUCLheasfr43qrf43rttq3";

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


function parseBookingWasmPointer(bookingPointer) {
    let booking = new bookingCache.Booking(
        bookingPointer.get_buchungsNummer(), bookingPointer.get_buchungsDatum(),
        bookingPointer.get_loginName(), bookingPointer.get_dauerDerBuchung(),
        bookingPointer.get_fahrzeugId(), bookingPointer.get_preisNetto(),
        bookingPointer.get_status(), bookingPointer.get_longitude(),
        bookingPointer.get_langtitude()
    )

    return booking;
}


// App
const app = express();
// Wird vom Fahrzeug aufgerufen
// Soll im Cache die aktuelle Postion speichern
// In einem Bestimmten Abstand wird dieser Call vom Fahrzeug selbständig aufgerufen
app.get('/test', async function (req, res) {
    try {
        let result = await bookingCacheInstance.check_and_get_booking_in_cache("admin", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJsb2dpbl9uYW1lIjoiYWRtaW4iLCJpc19hZG1pbiI6dHJ1ZSwiaWF0IjoxNjcwNjY3NDcwOTk4fQ.afC-Yah8VD5gXIJHftUetW3wzkUPo1_7Ca3HbgimcmQ", 1, circuitBreakerBuchungsverwaltung)
        if(result.get_buchungsNummer() > 0) {
            result.set_status("started");
            let booking = parseBookingWasmPointer(result);
            bookingCacheInstance.update_or_insert_cached_entrie(false, 0, booking);
            res.status(200).send(result.get_buchungsDatum());
        } else {
            res.status(400).send("Angegebene Buchung konnte nicht abgerufen werden!");
        }


    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }
});

app.get('/testCircuitBreakerWasm', async function (req, res) {
    try {
        let loginName = "admin";
        let authToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJsb2dpbl9uYW1lIjoiYWRtaW4iLCJpc19hZG1pbiI6dHJ1ZSwiaWF0IjoxNjcwNjY3NDcwOTk4fQ.afC-Yah8VD5gXIJHftUetW3wzkUPo1_7Ca3HbgimcmQ";
        let result = await circuitBreakerBuchungsverwaltungWasm.circuit_breaker_post_request("/getBooking/1", loginName, authToken, "GET", client);
        console.log(result);
        res.status(200).send(result);

    } catch(e){
        console.log("GJGBHJHSRGJHG");
        console.log(e.name, e.message);
        console.log("GJKSJRGNHKGJHNEJKRH")
        res.status(401).send("Something went Wrong ");
    }
});

app.get('/testCircuitBreakerWasmInput', async function (req, res) {
    try {
        let loginName = "admin";
        let authToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJsb2dpbl9uYW1lIjoiYWRtaW4iLCJpc19hZG1pbiI6dHJ1ZSwiaWF0IjoxNjcwNjY3NDcwOTk4fQ.afC-Yah8VD5gXIJHftUetW3wzkUPo1_7Ca3HbgimcmQ";
        let result = await circuitBreakerBuchungsverwaltungWasm.circuit_breaker_post_request("/getBooking/1", loginName, authToken, "GET", client);
        console.log(result);
        res.status(200).send(result);

    } catch(e){
        console.log("GJGBHJHSRGJHG");
        console.log(e.name, e.message);
        console.log("GJKSJRGNHKGJHNEJKRH")
        res.status(401).send("Something went Wrong ");
    }
});

app.get('/testBookingCacheCircuitBreaker', async function (req, res) {
    try {

        var bookingCache = require('./wasm_modules/bookingCacheAndCiruitBreaker/pkg/bookingCache.js');
        var circuitBreaker = require('./wasm_modules/bookingCacheAndCiruitBreaker/CircuitBrekaer.js');
        var httpClient = require('./wasm_modules/bookingCacheAndCiruitBreaker/httpClient.js');

        var bookingCacheInstanceWithCircuitBreaker = new bookingCache.Cache(BigInt(10000), BigInt(10000));
        var circuitBreakerInstance = new circuitBreaker.CircuitBreaker(BigInt(150), BigInt(30), BigInt(0),
            BigInt(-3), BigInt(10), BigInt(3), process.env.BUCHUNGSVERWALTUNG, 80)
        var client = new httpClient();


        let result = await bookingCacheInstanceWithCircuitBreaker.check_and_get_booking_in_cache("admin",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJsb2dpbl9uYW1lIjoiYWRtaW4iLCJpc19hZG1pbiI6dHJ1ZSwiaWF0IjoxNjcwNjY3NDcwOTk4fQ.afC-Yah8VD5gXIJHftUetW3wzkUPo1_7Ca3HbgimcmQ",
            1, circuitBreakerInstance, client)
        if(result.get_buchungsNummer() > 0) {
            result.set_status("started");
            let booking = parseBookingWasmPointer(result);
            bookingCacheInstanceWithCircuitBreaker.update_or_insert_cached_entrie(false, 0, booking);
            res.status(200).send(result.get_buchungsDatum());
        } else {
            res.status(400).send("Angegebene Buchung konnte nicht abgerufen werden!");
        }


    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }
});


app.listen(PORT, HOST, () => {
    console.log(`Running on http://${HOST}:${PORT}`);
});
