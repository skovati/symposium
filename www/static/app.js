const chat = document.getElementById('chatroom');
const input = document.getElementById('message-input');
const send = document.getElementById('message-send');
const uri = 'ws://' + location.host + '/ws';
const ws = new WebSocket(uri);

function message(data) {
    const line = document.createElement('p');
    line.innerText = data;
    chat.appendChild(line);
}

function keyPressed(event) {
    // if enter is pressed
    if (event.keyCode == 13) {
        sendMessage();
    }
}

function sendMessage() {
    const msg = input.value;
    ws.send(msg);
    input.value = '';
    chat.scrollTop = chat.scrollHeight;
};

ws.onopen = function() {
    chat.innerHTML = '<p><em>Connected to the Symposium Network!</em></p>';
};

ws.onmessage = function(msg) {
    message(msg.data);
};

ws.onclose = function() {
    chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
};

