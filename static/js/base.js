var connection = undefined;

let wsConnection = function wsConnection() {
    if (connection === undefined) {
        connection = new WebSocket('ws://' + window.location.hostname + ':9123/ws');
    }
    return connection;
}