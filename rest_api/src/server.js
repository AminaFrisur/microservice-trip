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

// App
const app = express();


// Wird vom Fahrzeug aufgerufen
// Soll im Cache die aktuelle Postion speichern
// In einem Bestimmten Abstand wird dieser Call vom Fahrzeug selbständig aufgerufen
app.get('/test', async function (req, res) {
    try {
        console.log("TEESSSTTT");
        bookingCacheInstance.check_and_get_booking_in_cache("admin", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJsb2dpbl9uYW1lIjoiYWRtaW4iLCJpc19hZG1pbiI6dHJ1ZSwiaWF0IjoxNjcwNTc4OTY5OTk4fQ.EYmPmuTDqrdeELnEE_17AnHKDiyfzVzR-BcZrsJfCuc", 1, circuitBreakerBuchungsverwaltung)
        console.log("TFDRIJVHIUDSHIHSDREDIDGSHF");
    } catch(err){
        console.log(err);
        res.status(401).send(err);
    }
});


app.listen(PORT, HOST, () => {
    console.log(`Running on http://${HOST}:${PORT}`);
});
