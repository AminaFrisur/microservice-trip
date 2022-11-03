class Cache {
    cachedEntries;
    timestamp;
    cacheTime;
    maxSize;

    // TODO: nicht das komplette Date speichern -> Viel zu speicher Intensiv

    // TODO: AUch später unbedingt erklären warum ich Array als Datenstruktur genommen habe und nicht beispielsweise linked list oder so

    // Cache Strategie:
    // wenn checkToken erfolgreich -> speichere Nutzer, Token, Token Timestamp in cachedEntries
    // Problem: Irgendwann läuft der Cache voll bzw. der Service ist mit der Datenmenge einfach überlastet
    // Meine Cache Strategie:
    // Jeder Cache Eintrag hat einen Timestamp
    // Bei jeder Nutzung wird dieser aktualisiert
    // Wenn der Timestamp dann älter ist als 5 Minuten -> schmeiße den Eintrag raus
    // Speicherung des Caches: nach Login Name in geordneter Reihenfolge
    // Ich nutze die Methode slice von Javascript array
    // diese macht eine shallow Copy : Die Referenzen bleiben gleich

    // cacheTime immer in Millisekunden angeben
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

}

module.exports = Cache