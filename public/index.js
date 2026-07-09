const form = document.getElementById("upload-form");
const status = document.getElementById("status");
const output = document.getElementById("response");

form.addEventListener("submit", async (event) => {
    event.preventDefault();

    const file = document.getElementById("file").files[0];
    if (!file) return;

    status.textContent = "TRANSMITTING... please wait";
    output.textContent = "";

    const body = new FormData();
    body.append("file", file);

    const response = await fetch(form.action, { method: "POST", body });
    const raw = await response.text();

    let parsed;
    try {
        parsed = JSON.parse(raw);
        output.textContent = JSON.stringify(parsed, null, 2);
    } catch {
        output.textContent = raw;
    }

    if (!response.ok) {
        status.textContent = "ERROR - upload failed (see OUTPUT)";
        if (!output.textContent) {
            output.textContent = `HTTP ${response.status} ${response.statusText}`;
        }
        return;
    }

    status.textContent = "COMPLETE - upload successful";
});