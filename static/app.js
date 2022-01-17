const chat = document.getElementById("chatroom");
const chatroom = document.getElementById("chatroom-div");
const header = document.getElementById("status-header")
const input = document.getElementById("message-input");
const send = document.getElementById("message-button");
const uri = "ws://" + location.host + "/ws";
const ws = new WebSocket(uri);

function message(data) {
    const line = document.createElement("li");
    line.innerText = data;
    chat.appendChild(line);
    chatroom.scrollTop = chatroom.scrollHeight;
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
};

ws.onopen = function() {
    header.innerHTML = "<h2><em>Connected to the Symposium Network!</em></h2>";
};

ws.onmessage = function(msg) {
    var parcel = JSON.parse(msg.data);
    message("[" + parcel.postmark + "] " + parcel.from.name + ": " + parcel.payload);
};

ws.onclose = function() {
    header.innerHTML = "<h2><em>Disconnected.</em></h2>";
};
