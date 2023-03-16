FROM node:18.12.1-alpine
LABEL description="Rest Api of Microservice Trip with Node"

# Set working directory
WORKDIR ./rest-api/

COPY src/auth.js src/httpClient.js src/package.json src/server.js ./
COPY src/wasm_modules/bookingCache/pkg/bookingCache.js ./wasm_modules/bookingCache/pkg/
COPY src/wasm_modules/bookingCache/pkg/bookingCache_bg.wasm ./wasm_modules/bookingCache/pkg/
COPY src/wasm_modules/bookingCache/target/wasm32-unknown-unknown/release/bookingCache.wasm ./wasm_modules/bookingCache/target/wasm32-unknown-unknown/release/
COPY src/wasm_modules/circuitBreaker/pkg/CircuitBrekaer.js ./wasm_modules/circuitBreaker/pkg/
COPY src/wasm_modules/circuitBreaker/pkg/CircuitBrekaer_bg.wasm ./wasm_modules/circuitBreaker/pkg/
COPY src/wasm_modules/circuitBreaker/target/wasm32-unknown-unknown/release/CircuitBrekaer.wasm ./wasm_modules/circuitBreaker/target/wasm32-unknown-unknown/release/
COPY src/wasm_modules/jwt/pkg/jwt.js ./wasm_modules/jwt/pkg/
COPY src/wasm_modules/jwt/pkg/jwt_bg.wasm ./wasm_modules/jwt/pkg/
COPY src/wasm_modules/jwt/target/wasm32-unknown-unknown/release/jwt.wasm ./wasm_modules/jwt/target/wasm32-unknown-unknown/release/

RUN npm install

CMD [ "node", "server.js" ]


