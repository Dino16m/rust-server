async function make_request() {
    const response = await fetch("http://localhost:4221");
    const text = await response.text();
    console.log(text);
}

async function stress(size) {
    const requests = [];
    for (let i = 0; i < size; i++) {
        requests.push(make_request());
    }
    const label = `${size} requests`;
    console.time(label);
    await Promise.all(requests);
    console.timeEnd(label);
}

stress(100);
