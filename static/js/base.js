var connection = undefined;

let wsConnection = function wsConnection() {
    if (connection === undefined) {
        connection = new WebSocket('ws://' + window.location.hostname + ':9123/ws');
    }
    return connection;
}

let copyToClipboard = function copyToClipboard(text) {
    var el = document.createElement('textarea');
    // Den zu kopierenden String dem Element zuweisen
    el.value = text;
    // Element nicht editierbar setzen und aus dem Fenster schieben
    el.setAttribute('readonly', '');
    el.style = {position: 'absolute', left: '-9999px'};
    document.body.appendChild(el);
    // Text innerhalb des Elements auswählen
    el.select();
    // Ausgewählten Text in die Zwischenablage kopieren
    document.execCommand('copy');
    // Temporäres Element löschen
    document.body.removeChild(el);
}
