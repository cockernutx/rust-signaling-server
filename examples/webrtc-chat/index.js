let stunServer = "ws://localhost:8081/ws";
let randomNum = Math.floor(Math.random() * 10);
console.log(stunServer);
let socket = new WebSocket(`${stunServer}`);
let myName = "";
let target = "";
let peerConnection = new RTCPeerConnection();

let channel ;



peerConnection.onicecandidate = (e) => {

  if (e.candidate) {
    let iceCandidate = {
      type: "new_ice_candidate",
      data: {
        target: target,
        candidate: JSON.stringify(e.candidate.candidate),
        name: myName
      }
    }

    socket.send(JSON.stringify(iceCandidate))
  }
}

peerConnection.onconnectionstatechange = (state) => {
  if (peerConnection.iceConnectionState === "connected") {
    alert("Connection open! You can now send messages.")

  }

}

peerConnection.ondatachannel = event => {
  channel = event.channel;
  assingChannelFunctions()
};

socket.onmessage = function (event) {
  console.log("Received data from server: " + event.data);
  let data = JSON.parse(event.data);


  switch (data.type) {
    case "assign":
      myName = data.data.name;
      document.getElementById("name").innerText = `You name is: ${myName}`;
      break;
    case "offer":
      sendAnswer(data);
      break;
    case "answer":
      handleAnswer(data)
      break;
    case "new_ice_candidate":
      addIceCandidate(data)
      break;
  }
};

socket.onclose = function (event) {
  if (event.wasClean) {
    alert(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
  } else {
    // e.g. server process killed or network down
    // event.code is usually 1006 in this case
    alert('[close] Connection died');
  }
};

socket.onerror = function (error) {
  alert(`[error]`);
};

async function sendOffer() {
  const dataChannelParams = { ordered: true };
  channel = peerConnection.createDataChannel('messaging-channel', dataChannelParams);
  assingChannelFunctions()
  target = document.getElementById("target").value;
  let offerOptions = {

  };
  let offer = await peerConnection.createOffer();
  await peerConnection.setLocalDescription(offer)

  console.log(offer.sdp);

  let sdp = offer.sdp;
  let offerWithTarget = JSON.stringify({
    type: offer.type,
    data: {
      name: myName,
      target: target,
      sdp: sdp
    }

  });




  socket.send(offerWithTarget);

  console.log(`Send offer: ${offerWithTarget}`);
}

async function sendAnswer(offer) {

  target = offer.data.name;

  let offerOptions = {

  };
  await peerConnection.setRemoteDescription({
    type: offer.type,
    target: offer.data.target,
    sdp: offer.data.sdp
  });
  let answer = await peerConnection.createAnswer();
  await peerConnection.setLocalDescription(answer)

  let answerWithTarget = JSON.stringify({
    type: answer.type,
    data: {
      name: myName,
      target: target,
      sdp: answer.sdp
    }

  });

  socket.send(answerWithTarget);

  console.log(`Send ansewr: ${answerWithTarget}`);
}

async function handleAnswer(data) {
  await peerConnection.setRemoteDescription({
    type: data.type,
    target: data.data.target,
    sdp: data.data.sdp
  });


}

async function addIceCandidate(data) {
  let candidate = JSON.parse(data.data.candidate);
  await peerConnection.addIceCandidate(candidate);
}


function assingChannelFunctions() {
  channel.onopen = (ev) => {
    console.log("channel open")
  }
  
  channel.onmessage = (msg) => {
    alert(`new message: ${msg.data}`)
  }

  channel.onclose = (ev) => {
    console.log("channel closed")
  }
}

function send() {
  let msg = document.getElementById("message").value;
  channel.send(msg)
}