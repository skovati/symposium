async function checkUsername() {
    let url = window.location.href + "register";
    const msg = document.getElementById("login-header");
    const username = document.getElementById("username-input");
    const data = {
        name: username.value,
        id: "",
    }
    const response = await postUserForm(url, data);
    if(response.ok) {
        msg.innerHTML = "<h2><em>Forwarding to chat page!</em></h2>";
        const user = await response.json();
        sessionStorage.setItem("name", user.name);
        sessionStorage.setItem("id", user.id);
        window.location.replace("chat");
    } else {
        msg.innerHTML = "<h2><em>Sorry that username is taken, try again</em></h2>";
        return;
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
