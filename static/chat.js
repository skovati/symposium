const chat = document.getElementById("chatroom");
const chatroom = document.getElementById("chatroom-div");
const header = document.getElementById("status-header")
const input = document.getElementById("message-input");
const send = document.getElementById("message-button");
let ws;
let username;

function connect() {
    let url = new URL(
        (window.location.protocol === 'https:' ? 'wss://' : 'ws://') +
        window.location.host +
        '/ws/')
    const name = sessionStorage.getItem("name");
    const id = sessionStorage.getItem("id");
    url.searchParams.append('name', name);
    url.searchParams.append('id', id);

    username = name;
    input.select();

    ws = new WebSocket(url);

    ws.onerror = function() {
        window.location.replace("/");
    }

    ws.onopen = function() {
        header.innerHTML = "<h2><em>Connected to the Symposium Network!</em></h2>";
    };

    ws.onmessage = function(msg) {
        let parcel = JSON.parse(msg.data);
        let data;
        if(parcel.from.name == username) {
            data = "[" + parcel.postmark + "] " + '<span style="color: firebrick">' + parcel.from.name + '</span>' + ": " + parcel.payload;
        }
        else {
            data = "[" + parcel.postmark + "] " + parcel.from.name + ": " + parcel.payload;
        }

        const line = document.createElement("li");
        line.innerHTML = data;
        chat.appendChild(line);
        chatroom.scrollTop = chatroom.scrollHeight;
    };

    ws.onclose = function() {
        header.innerHTML = "<h2><em>Disconnected.</em></h2>";
    };
}

function keyPressed(event) {
    // if enter is pressed
    if (event.keyCode == 13) {
        sendMessage();
    }
}

function sendMessage() {
    const msg = input.value.trim();
    if(msg) {
        ws.send(msg);
        input.value = "";
    }
};

window.onload = connect;
