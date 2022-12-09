var jwt = require('./wasm_modules/jwt/pkg/jwt.js');
module.exports = function() {
    var module = {};
    module.checkAuth = async function(req, res, isAdmin, jwt_secret, next) {
        let authToken = req.headers.auth_token;
        let loginName = req.headers.login_name;

        try {
            let check = jwt.jwt_sign(loginName, authToken, isAdmin, jwt_secret);
            if(check) {
                console.log("AUTH: token ist Valide");
                next();
            } else {
                res.status(401).send("token is not valid");
            }
        } catch(e) {
            console.log("AUTH: " + e)
            res.status(401).send("token and/or login name are missing or are not valid");
        }

    }

    return module;
}
