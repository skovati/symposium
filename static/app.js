const chat = document.getElementById("chatroom");
const chatroom = document.getElementById("chatroom-div");
const header = document.getElementById("status-header")
const input = document.getElementById("message-input");
const send = document.getElementById("message-button");
const uri = "ws://" + location.host + "/ws";
const ws = new WebSocket(uri);

function checkUsername() {
    console.log("test");
    postUserForm();
}

async function postUserForm() {
    const msg = document.getElementById("login-header");
    const username = document.getElementById("username-input").value;
    const requestOptions = {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: username })
    };
    fetch("http://127.0.0.1:8080/register", requestOptions)
        .then(response => response.json())
        .then(data => msg.innerHTML = data );
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
