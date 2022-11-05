const Cache = require('./cache.js');
class BookingCache extends Cache {

    // prüfe ob das Token noch im Gültigkeitszeitraum liegt
    // wenn nicht dann muss ein neues Token vom Microservice Benutzerverwaltung angefordert werden
    async checkAndGetBookingInCache(loginName, authToken, buchungsNummer, circuitBreaker) {
        // User wurde gefunden, prüfe nun token und Timestamp vom token
        let index = this.getCacheEntryIndex("buchungsNummer", buchungsNummer);
        if(index >= 0) {
            if(loginName != this.cachedEntries[index].loginName) {
                console.log("Cache Booking: übergebener LoginName entpricht nicht dem aus dem Cache");
                console.log("Cache Booking: Zugriff auf die Buchung ist nicht erlaubt");
                throw "Zugriff auf die Buchung "+ buchungsNummer + " ist vom Nutzer "+ loginName + " nicht erlaubt!";
            } else {
                return {"booking": this.cachedEntries[index], "index": index};
            }
        } else {

            // Buchung ist nicht im cache
            // Also mache einen Request auf den Microservice Buchungsverwaltung
            let headerData = {
                'auth_token': authToken,
                'login_name': loginName
            };
            console.log("BookingCache: HeaderDaten = " + authToken + " und " + loginName)
            let response = await circuitBreaker.circuitBreakerRequest("/getBooking/" + buchungsNummer, "", headerData, "GET");
            console.log("BookingCache: response ist");
            console.log(response);
            if(response) {
                let booking = {
                    "buchungsNummer": response[0].buchungsNummer,
                    "buchungsDatum": response[0].buchungsDatum,
                    "loginName": response[0].loginName,
                    "fahrzeugId": response[0].fahrzeugId,
                    "dauerDerBuchung": response[0].dauerDerBuchung,
                    "preisNetto": response[0].preisNetto,
                    "status": response[0].status
                }
                // Wenn erfolgreich, speichere Buchung in den Cache
                return {"booking": booking, "index": -1};
            } else {
                return false;
            }

        }

    }

}

module.exports = BookingCache