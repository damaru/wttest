const CERT_HASH = new Uint8Array(${CERT_DIGEST});
const DEFAULT_URL = "${WEBTRANSPORT_URL}";

let transport;
let datagramWriter;
let streamNumber = 1;

window.addEventListener("DOMContentLoaded", () => {
  document.getElementById("url").value = DEFAULT_URL;
  document.getElementById("connect").addEventListener("click", connect);
  document.getElementById("send").addEventListener("click", sendMessage);
});

async function connect() {
  const url = document.getElementById("url").value;

  try {
    transport = new WebTransport(url, {
      serverCertificateHashes: [{
        algorithm: "sha-256",
        value: CERT_HASH.buffer,
      }],
    });
  } catch (error) {
    log(`Failed to create WebTransport: ${error}`, "error");
    return;
  }

  log("Connecting...");

  try {
    await transport.ready;
    log("Connected");
  } catch (error) {
    log(`Connection failed: ${error}`, "error");
    return;
  }

  transport.closed
    .then(() => log("Connection closed"))
    .catch((error) => log(`Connection closed abruptly: ${error}`, "error"));

  datagramWriter = transport.datagrams.writable.getWriter();
  readDatagrams(transport);
  acceptIncomingUnidirectionalStreams(transport);

  document.getElementById("connect").disabled = true;
  document.getElementById("send").disabled = false;
}

async function sendMessage() {
  const text = document.getElementById("message").value;
  const data = new TextEncoder().encode(text);
  const type = document.querySelector("input[name='send-type']:checked").value;

  try {
    if (type === "datagram") {
      await datagramWriter.write(data);
      log(`Sent datagram: ${text}`);
      return;
    }

    if (type === "bidi") {
      const stream = await transport.createBidirectionalStream();
      const number = streamNumber++;
      readStream(stream.readable, number);
      const writer = stream.writable.getWriter();
      await writer.write(data);
      await writer.close();
      log(`Sent bidirectional stream #${number}: ${text}`);
      return;
    }

    const stream = await transport.createUnidirectionalStream();
    const writer = stream.getWriter();
    await writer.write(data);
    await writer.close();
    log(`Sent unidirectional stream: ${text}`);
  } catch (error) {
    log(`Send failed: ${error}`, "error");
  }
}

async function readDatagrams(currentTransport) {
  const reader = currentTransport.datagrams.readable.getReader();
  const decoder = new TextDecoder();

  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) return;
      log(`Received datagram: ${decoder.decode(value)}`);
    }
  } catch (error) {
    log(`Datagram read failed: ${error}`, "error");
  }
}

async function acceptIncomingUnidirectionalStreams(currentTransport) {
  const reader = currentTransport.incomingUnidirectionalStreams.getReader();

  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) return;
      const number = streamNumber++;
      log(`Incoming unidirectional stream #${number}`);
      readStream(value, number);
    }
  } catch (error) {
    log(`Incoming stream read failed: ${error}`, "error");
  }
}

async function readStream(stream, number) {
  const reader = stream.pipeThrough(new TextDecoderStream()).getReader();

  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) {
        log(`Stream #${number} closed`);
        return;
      }
      log(`Received on stream #${number}: ${value}`);
    }
  } catch (error) {
    log(`Stream #${number} read failed: ${error}`, "error");
  }
}

function log(message, level = "info") {
  const list = document.getElementById("log");
  const item = document.createElement("li");
  item.textContent = message;
  item.className = level;
  list.appendChild(item);
  item.scrollIntoView();
}
