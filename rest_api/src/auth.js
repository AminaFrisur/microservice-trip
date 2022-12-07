var jwt = require('jsonwebtoken');
module.exports = function() {
    var module = {};
    module.checkAuth = async function(req, res, isAdmin, jwt_secret, next) {
        let authToken = req.headers.auth_token;
        let loginName = req.headers.login_name;

        try {
            var decoded = jwt.verify(authToken, jwt_secret);
            console.log(decoded);
            if(decoded && decoded.iat && decoded.login_name == loginName &&
                ((isAdmin && decoded.is_admin == true) || isAdmin == false)) {

                // check timestamp
                let timeDiff = new Date() - decoded.iat;
                console.log("AUTH: TimeDiff von Token ist: " + timeDiff);
                if(timeDiff > 2000000) {
                    console.log("AUTH: Auth Token ist zu alt")
                    res.status(401).send("token is to old. Please get a new one.");
                } else {
                    console.log("Authentifizierungstoken ist valide");
                    next();
                }


            } else {
                res.status(401).send("token and/or login name are missing or are not valid");
            }
        } catch(e) {
            console.log("AUTH: " + e)
            res.status(401).send("token and/or login name are missing or are not valid");
        }

    }

    return module;
}
