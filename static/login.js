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
        msg.innerHTML = "<h2><em>Enter a username!</em></h2>";
        console.log(await response.text());
    } else {
        console.log("invalid username");
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
