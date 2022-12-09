# microservice-trip
Micorservice Trip mit Wasm


Zu Wasm-Bindgen:
- nicht jede ist damit Kompatibel:
- Beispiel:  the trait `ReturnWasmAbi` is not implemented for `Result<(), anyhow::Error>
- Ebenfalls für Tupel: the trait `IntoWasmAbi` is not implemented for `(bool, std::string::String)`
- the trait `IntoWasmAbi` is not implemented for `jwt_claims` -> Structs können nicht als Rückgabe Parameter verwendet werden
- Auch nicht als Eingabe Parameter: the trait `FromWasmAbi` is not implemented for `jwt_claims`
- wenn panic! oder unwrap -> RuntimeError: unreachable
