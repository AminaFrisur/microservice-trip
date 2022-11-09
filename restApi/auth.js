export async function checkAuth(isAdmin, loginName, authToken, userCache, circuitBreaker) {
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
            let loginData = JSON.parse(await circuitBreaker.circuitBreakerRequest("/checkAuthUser", bodyData, headerData, "POST"));
            console.log("Authentification: Request checkAuthUser ergab folgendes Ergebnis: " + loginData);
            if(loginData && loginData[0]) {
                var user = {
                    "loginName": loginName,
                    "authToken": loginData[0].auth_token,
                    "authTokenTimestamp": loginData[0].auth_token_timestamp,
                    "isAdmin": loginData[0].is_admin
                }
                userCache.updateOrInsertcachedEntrie(userIndexinCache, user);
                return;
            } else {
                console.log("Authentification: Token ist laut Benutzerverwaltung nicht valide");
                throw "token and/or login name are missing or are not valid";
            }
        } catch(e) {
            console.log("Authentification: Authentifizierung des Nutzer schlug fehl -> " + e);
            throw "Authentifizierung des Nutzer schlug fehl -> " + e;
        }
    } else {
        console.log("Authentification: Nutzer ist noch zwischengespeichert");
        return;
    }

}