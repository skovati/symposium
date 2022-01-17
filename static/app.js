const chat = document.getElementById("chatroom");
const chatroom = document.getElementById("chatroom-div");
const header = document.getElementById("status-header")
const input = document.getElementById("message-input");
const send = document.getElementById("message-button");
let ws;

function connect() {
    let url = new URL(
        (window.location.protocol === 'https:' ? 'wss://' : 'ws://') +
        window.location.host +
        '/ws/')
    const name = sessionStorage.getItem("name");
    const id = sessionStorage.getItem("id");
    url.searchParams.append('name', name);
    url.searchParams.append('id', id);

    ws = new WebSocket(url);
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
}

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

window.onload = connect;
