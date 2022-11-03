const Cache = require('./cache.js');
class UserCache extends Cache {

    // prüfe ob das Token noch im Gültigkeitszeitraum liegt
    // wenn nicht dann muss ein neues Token vom Microservice Benutzerverwaltung angefordert werden
    checkToken(index, authToken, isAdmin) {
        // User wurde gefunden, prüfe nun token und Timestamp vom token
        if(index >= 0) {
            if(authToken != this.cachedEntries[index].authToken) {console.log("Cache: Token aus dem Header stimmt nicht mit dem Token aus dem cache überein"); return false};
            if(this.cachedEntries[index].isAdmin != true && isAdmin == true) {console.log("Cache: isAdmin ist false"); return false};

            // Rechne von Millisekunden auf Stunden um
            let timeDiff = (new Date() - this.cachedEntries[index].authTokenTimestamp) / 3600000;
            // Wenn token älter ist als 24 Stunden
            if(timeDiff > 24) {
                return false;
            }
            return true;
        } else {
            return false;
        }
    }

}

module.exports = UserCache