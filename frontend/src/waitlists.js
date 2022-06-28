export async function processWaitlists(callback) {
    const wlRaw = await fetch("/api/waitlists");
    if (wlRaw.status !== 200) {
        callback(null);
        return;
    }
    const wlJson = await wlRaw.json();
    callback(wlJson);
}
