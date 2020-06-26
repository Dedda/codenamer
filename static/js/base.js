var connection = undefined;

let wsConnection = function wsConnection() {
    if (connection === undefined) {
        connection = new WebSocket('ws://' + window.location.hostname + ':9123/ws');
    }
    return connection;
}

let copyToClipboard = function copyToClipboard(text) {
    var el = document.createElement('textarea');
    el.value = text;
    el.setAttribute('readonly', '');
    el.style = {position: 'absolute', left: '-9999px'};
    document.body.appendChild(el);
    el.select();
    document.execCommand('copy');
    document.body.removeChild(el);
}
