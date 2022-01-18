const msg = document.getElementById("login-header");
const input = document.getElementById("username-input");

async function checkUsername() {
    let url = window.location.href + "register";
    const username = input.value.trim();
    if(username) {
        const data = {
            name: username,
            id: "",
        }
        const response = await postUserForm(url, data);
        if(response.ok) {
            msg.innerHTML = "<h2><em>Forwarding to chat page!</em></h2>";
            const user = await response.json();
            sessionStorage.setItem("name", user.name);
            sessionStorage.setItem("id", user.id);
            window.location.assign("chat");
        } else {
            msg.innerHTML = "<h2><em>Sorry that username is taken, try again</em></h2>";
            return;
        }
    }
}

async function postUserForm(url, data) {
    const resp = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify(data)
    });
    return resp;
}

function keyPressed(event) {
    // if enter is pressed
    if (event.keyCode == 13) {
        checkUsername();
    }
}

window.addEventListener('load',function(){
    input.select();
})
