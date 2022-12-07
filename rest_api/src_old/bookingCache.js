class BookingCache {

    cachedEntries;
    timestamp;
    cacheTime;
    maxSize;

    constructor(cacheTime, maxSize) {
        this.cachedEntries = new Array();
        this.timestamp = new Date();
        this.cacheTime = cacheTime;
        this.maxSize = maxSize;
    }

    clearCache() {
        // Map speichert in Insertion Order
        console.log("Cache: Prüfe ob Einträge aus dem Cache gelöscht werden können");
        if (this.cachedEntries.size > this.maxSize) {
            // kompletter reset des caches
            // sollte aber eigentlich nicht passieren
            this.cachedEntries = [];
            return;
        }

        let tempIndex = this.cachedEntries.length
        let check = true;

        while (check) {
            tempIndex = parseInt(tempIndex / 2);
            console.log("Cache: TempIndex ist " + tempIndex);
            // Falls im Cache nur ein Element ist
            if (tempIndex >= 1) {

                // Array ist größer als 1 also mache teile und hersche
                console.log(this.cachedEntries[tempIndex - 1]);
                let timeDiff = new Date() - this.cachedEntries[tempIndex - 1].cacheTimestamp;
                console.log(timeDiff - this.cacheTime);
                // Wenn für den Eintrag die Cache Time erreicht ist -> lösche die hälfte vom Part des Arrays was betrachtet wird
                // Damit sind dann nicht alle alten Cache einträge gelöscht -> aber das clearen vom Cache sollte schnell gehen
                if (timeDiff >= this.cacheTime) {
                    console.log("Cache: Clear Cache");
                    this.cachedEntries = [
                        ...this.cachedEntries.slice(tempIndex)
                    ]
                    check = false;
                }

                // Wenn timeDiff noch stimmt dann mache weiter

            } else {

                // auch wenn das eine Element im Array ein alter Eintrag ist
                // kann dies vernachlässigt werden bzw. ist nicht so wichtig
                console.log("Cache: nichts zu clearen")
                check = false;
            }


        }

        console.log(this.cachedEntries);
    }

    getCacheEntryIndex(searchParam, searchParamValue) {
        // an dieser Stelle erst den Cache leeren
        // wenn clearCache an andere Stelle aufgerufen wird, dann stimmt der Index nicht mehr
        this.clearCache();
        let finalIndex = -1;
        // O(N) -> Aufwand bei jedem cache durchlauf
        for (var i = 0; i < this.cachedEntries.length; i++) {
            console.log(this.cachedEntries[i][searchParam]);
            if(this.cachedEntries[i][searchParam] == searchParamValue) {
                finalIndex = i;
                // Auch beim Suchen eines Users -> Timestamp für Cache Eintrag aktualisieren
                console.log("Cache: Update Timestamp vom Cache Eintrag");
                this.cachedEntries[i].cacheTimestamp = new Date();
                break;
            }
        }
        console.log("Cache: User Index ist:" + finalIndex);
        return finalIndex;
    }


    updateOrInsertcachedEntrie(index, newCacheEntry) {

        newCacheEntry.cacheTimestamp = new Date();

        if(index >= 0 ) {
            // update Nutzer
            console.log("Abstract Cache: mache ein Update");
            this.cachedEntries = [
                ...this.cachedEntries.slice(0, index),
                ...this.cachedEntries.slice(index + 1)
            ]
            console.log("Cache: update User Cache");
            this.cachedEntries.push(newCacheEntry);
            console.log(this.cachedEntries);
        } else {
            // Füge User neu im Cache hinzu, da nicht im cache vorhanden
            console.log("Abstract Cache: Füge neuen Eintrag in Cache hinzu");
            this.cachedEntries.push(newCacheEntry);
            console.log(this.cachedEntries);
        }
    }

    removeFromCache(index) {
        this.cachedEntries = [
            ...this.cachedEntries.slice(0, index),
            ...this.cachedEntries.slice(index + 1)
        ]
    }

    getAllCacheEntrys() {
        return this.cachedEntries;
    }


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
            if(response && response.length > 0) {
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