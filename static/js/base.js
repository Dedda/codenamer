var connection = undefined;

let wsConnection = function wsConnection(url) {
    if (connection === undefined) {
        connection = new WebSocket(url);
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
