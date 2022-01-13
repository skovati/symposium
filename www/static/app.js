const chat = document.getElementById('chat');
const text = document.getElementById('text');
const uri = 'ws://' + location.host + '/ws';
const ws = new WebSocket(uri);


function message(data) {
    const line = document.createElement('p');
    line.innerText = data;
    chat.appendChild(line);
}

ws.onopen = function() {
    chat.innerHTML = '<p><em>Connected to the Symposium Network!</em></p>';
};

ws.onmessage = function(msg) {
    message(msg.data);
};

ws.onclose = function() {
    chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
};

send.onclick = function() {
    const msg = text.value;
    ws.send(msg);
    text.value = '';
    // message('[you]: ' + msg);
};
