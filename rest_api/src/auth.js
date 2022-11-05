module.exports = function() {
    var module = {};
    // mache ein Post Request und Frage login Token ab
    // Falls nicht vorhanden kann die Buchung nicht vorgenommen werden
    // Bei jedem Zugriff auf Booking muss über den HTTP Header ein Auth Token zurückgegeben werden
    // Anschließend erfolgt eine Abfrage an MS Benutzerverwaltung
    // Booking MS speichert bei erfolg diesen zwischen (Key Value Store) mit den Parametern auth Token, Datum, Login Name

     module.checkAuth = async function(req, res, isAdmin, userCache, circuitBreaker, next) {
        let authToken = req.headers.auth_token;
        let loginName = req.headers.login_name;

        // Schritt 1: Schaue ob der User im userCache ist
         // Hier aufgerufen um nur einmal getUserIndex aufzurufen
        let userIndexinCache = userCache.getCacheEntryIndex("loginName", loginName);

        // Schritt 2: Prüfe ob auth Token im cache ist, übereinstimmt mit dem Token im Header und noch im Gültigkeitszeitraum liegt
        let check = userCache.checkToken(userIndexinCache, authToken, isAdmin);
        if(check == false) {
            // Schritt 2: Token ist nicht valide, Timestamp zu alt oder Auth Daten sind nicht im cache
            let bodyData = {"login_name":loginName, "auth_token": authToken, "isAdmin": isAdmin};
            let headerData = { 'Content-Type': 'application/json'};
            try {
                let loginData = await circuitBreaker.circuitBreakerRequest("/checkAuthUser", bodyData, headerData, "POST");
                console.log("Authentification: Request checkAuthUser ergab folgendes Ergebnis: " + loginData);
                // TODO: Mal überlegen ob das wirklich so RAW von der Benutzerverwaltung übergeben werden soll
                if(loginData) {
                    var user = {
                        "loginName": loginName,
                        "authToken": loginData[0].auth_token,
                        "authTokenTimestamp": loginData[0].auth_token_timestamp,
                        "isAdmin": loginData[0].is_admin
                    }
                    userCache.updateOrInsertcachedEntrie(userIndexinCache, user);
                    next();
                } else {
                    console.log("Authentification: Token ist laut Benutzerverwaltung nicht valide");
                    res.status(401).send("token and/or login name are missing or are not valid");
                }
            } catch(e) {
                console.log("Authentification: Reqeust schlug fehl ->" + e);
                res.status(401).send("Request zur Benutzerverwaltung schlug fehl!!");
            }
        } else {
            console.log("Authentification: Nutzer ist noch zwischengespeichert");
            next();
        }

    }
    return module;
}
