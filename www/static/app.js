const chat = document.getElementById("chatroom");
const chatroom = document.getElementById("chatroom-div");
const header = document.getElementById("status-header")
const input = document.getElementById("message-input");
const send = document.getElementById("message-send");
const uri = "ws://" + location.host + "/ws";
const ws = new WebSocket(uri);

function message(data) {
    const line = document.createElement("li");
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
    input.value = "";
    chatroom.scrollTop = chatroom.scrollHeight;
};

ws.onopen = function() {
    header.innerHTML = "<h2><em>Connected to the Symposium Network!</em></h2>";
};

ws.onmessage = function(msg) {
    message(msg.data);
};

ws.onclose = function() {
    header.innerHTML = "<h2><em>Disconnected.</em></h2>";
};

