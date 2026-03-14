#!/usr/bin/env node
// WebSocket relay server for s1engine real-time collaboration.
//
// Usage:
//   node scripts/relay.js [--port 8787]
//
// Protocol:
//   All messages are JSON. Each message has a "type" field.
//
//   Client → Server:
//     { type: "join",      room: "abc", userName: "Alice", userColor: "#e06c75" }
//     { type: "op",        room: "abc", data: <serialized CRDT op JSON> }
//     { type: "awareness", room: "abc", data: <serialized awareness update JSON> }
//     { type: "sync-req",  room: "abc", stateVector: <JSON state vector> }
//     { type: "leave",     room: "abc" }
//
//   Server → Client:
//     { type: "joined",    room: "abc", peerId: "...", peers: [...] }
//     { type: "peer-join", room: "abc", peerId: "...", userName: "...", userColor: "..." }
//     { type: "peer-leave",room: "abc", peerId: "..." }
//     { type: "op",        room: "abc", peerId: "...", data: <CRDT op JSON> }
//     { type: "awareness", room: "abc", peerId: "...", data: <awareness JSON> }
//     { type: "sync-resp", room: "abc", ops: [<array of CRDT op JSONs>] }
//     { type: "error",     message: "..." }
//
// Dependencies: ws (npm install ws)
//   If ws is not installed, falls back to a minimal raw WebSocket server.

const http = require('http');
const crypto = require('crypto');

const PORT = parseInt(process.argv.find((_, i, a) => a[i - 1] === '--port') || '8787', 10);

// ─── Room State ───────────────────────────────────────

// rooms: Map<roomId, { peers: Map<peerId, { ws, userName, userColor }>, opLog: string[] }>
const rooms = new Map();

function getOrCreateRoom(roomId) {
  if (!rooms.has(roomId)) {
    rooms.set(roomId, { peers: new Map(), opLog: [] });
  }
  return rooms.get(roomId);
}

function cleanupRoom(roomId) {
  const room = rooms.get(roomId);
  if (room && room.peers.size === 0) {
    // Keep room alive for 5 minutes after last peer leaves (for reconnection)
    room._cleanupTimer = setTimeout(() => {
      if (room.peers.size === 0) {
        rooms.delete(roomId);
      }
    }, 5 * 60 * 1000);
  }
}

// ─── Peer Management ──────────────────────────────────

function generatePeerId() {
  return crypto.randomBytes(8).toString('hex');
}

function broadcast(room, message, excludePeerId) {
  const json = JSON.stringify(message);
  for (const [pid, peer] of room.peers) {
    if (pid !== excludePeerId && peer.ws.readyState === 1) {
      try { peer.ws.send(json); } catch (_) { /* ignore send errors */ }
    }
  }
}

function sendTo(ws, message) {
  if (ws.readyState === 1) {
    try { ws.send(JSON.stringify(message)); } catch (_) {}
  }
}

// ─── Message Handlers ─────────────────────────────────

function handleJoin(ws, peerId, msg) {
  const roomId = msg.room;
  if (!roomId || typeof roomId !== 'string') {
    sendTo(ws, { type: 'error', message: 'Missing room ID' });
    return;
  }

  const room = getOrCreateRoom(roomId);
  if (room._cleanupTimer) {
    clearTimeout(room._cleanupTimer);
    room._cleanupTimer = null;
  }

  const userName = msg.userName || 'Anonymous';
  const userColor = msg.userColor || '#999999';

  room.peers.set(peerId, { ws, userName, userColor });
  ws._peerId = peerId;
  ws._roomId = roomId;

  // Send join confirmation with current peer list
  const peers = [];
  for (const [pid, peer] of room.peers) {
    if (pid !== peerId) {
      peers.push({ peerId: pid, userName: peer.userName, userColor: peer.userColor });
    }
  }
  sendTo(ws, { type: 'joined', room: roomId, peerId, peers });

  // Notify existing peers
  broadcast(room, {
    type: 'peer-join',
    room: roomId,
    peerId,
    userName,
    userColor,
  }, peerId);

  // Send buffered op log to new peer for catch-up
  if (room.opLog.length > 0) {
    sendTo(ws, { type: 'sync-resp', room: roomId, ops: room.opLog });
  }

  console.log(`[${roomId}] ${userName} joined (${room.peers.size} peers)`);
}

function handleOp(ws, peerId, msg) {
  const roomId = msg.room;
  const room = rooms.get(roomId);
  if (!room || !room.peers.has(peerId)) {
    sendTo(ws, { type: 'error', message: 'Not in room' });
    return;
  }

  // Store op in room log (cap at 10000 to prevent memory issues)
  if (typeof msg.data === 'string') {
    room.opLog.push(msg.data);
    if (room.opLog.length > 10000) {
      room.opLog.splice(0, room.opLog.length - 5000);
    }
  }

  // Broadcast to all other peers
  broadcast(room, {
    type: 'op',
    room: roomId,
    peerId,
    data: msg.data,
  }, peerId);
}

function handleAwareness(ws, peerId, msg) {
  const roomId = msg.room;
  const room = rooms.get(roomId);
  if (!room || !room.peers.has(peerId)) return;

  broadcast(room, {
    type: 'awareness',
    room: roomId,
    peerId,
    data: msg.data,
  }, peerId);
}

function handleSyncReq(ws, peerId, msg) {
  const roomId = msg.room;
  const room = rooms.get(roomId);
  if (!room) {
    sendTo(ws, { type: 'sync-resp', room: roomId, ops: [] });
    return;
  }

  // Return full op log — the CRDT will handle deduplication
  sendTo(ws, { type: 'sync-resp', room: roomId, ops: room.opLog });
}

function handleLeave(ws, peerId, msg) {
  const roomId = msg.room || ws._roomId;
  const room = rooms.get(roomId);
  if (!room) return;

  const peer = room.peers.get(peerId);
  const userName = peer ? peer.userName : 'Unknown';

  room.peers.delete(peerId);

  broadcast(room, {
    type: 'peer-leave',
    room: roomId,
    peerId,
  });

  console.log(`[${roomId}] ${userName} left (${room.peers.size} peers)`);
  cleanupRoom(roomId);
}

function handleDisconnect(ws) {
  const peerId = ws._peerId;
  const roomId = ws._roomId;
  if (peerId && roomId) {
    handleLeave(ws, peerId, { room: roomId });
  }
}

// ─── WebSocket Server (minimal, no dependencies) ──────

function acceptWebSocket(req, socket, head) {
  const key = req.headers['sec-websocket-key'];
  if (!key) { socket.destroy(); return; }

  const accept = crypto
    .createHash('sha1')
    .update(key + '258EAFA5-E914-47DA-95CA-5AB5A0085CC1')
    .digest('base64');

  socket.write(
    'HTTP/1.1 101 Switching Protocols\r\n' +
    'Upgrade: websocket\r\n' +
    'Connection: Upgrade\r\n' +
    `Sec-WebSocket-Accept: ${accept}\r\n` +
    '\r\n'
  );

  const ws = createWsWrapper(socket);
  const peerId = generatePeerId();
  ws._peerId = peerId;

  ws.on('message', (data) => {
    try {
      const msg = JSON.parse(data);
      switch (msg.type) {
        case 'join':      handleJoin(ws, peerId, msg); break;
        case 'op':        handleOp(ws, peerId, msg); break;
        case 'awareness': handleAwareness(ws, peerId, msg); break;
        case 'sync-req':  handleSyncReq(ws, peerId, msg); break;
        case 'leave':     handleLeave(ws, peerId, msg); break;
        default:
          sendTo(ws, { type: 'error', message: `Unknown message type: ${msg.type}` });
      }
    } catch (e) {
      sendTo(ws, { type: 'error', message: 'Invalid JSON' });
    }
  });

  ws.on('close', () => handleDisconnect(ws));
}

// Minimal WebSocket frame parser/writer
function createWsWrapper(socket) {
  const events = {};
  let closed = false;

  const ws = {
    readyState: 1,
    _peerId: null,
    _roomId: null,

    on(event, handler) {
      if (!events[event]) events[event] = [];
      events[event].push(handler);
    },

    send(data) {
      if (closed) return;
      const buf = Buffer.from(data, 'utf8');
      let header;
      if (buf.length < 126) {
        header = Buffer.alloc(2);
        header[0] = 0x81; // FIN + text
        header[1] = buf.length;
      } else if (buf.length < 65536) {
        header = Buffer.alloc(4);
        header[0] = 0x81;
        header[1] = 126;
        header.writeUInt16BE(buf.length, 2);
      } else {
        header = Buffer.alloc(10);
        header[0] = 0x81;
        header[1] = 127;
        header.writeBigUInt64BE(BigInt(buf.length), 2);
      }
      socket.write(Buffer.concat([header, buf]));
    },
  };

  let buffer = Buffer.alloc(0);

  socket.on('data', (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);

    while (buffer.length >= 2) {
      const opcode = buffer[0] & 0x0f;
      const masked = (buffer[1] & 0x80) !== 0;
      let payloadLen = buffer[1] & 0x7f;
      let offset = 2;

      if (payloadLen === 126) {
        if (buffer.length < 4) return;
        payloadLen = buffer.readUInt16BE(2);
        offset = 4;
      } else if (payloadLen === 127) {
        if (buffer.length < 10) return;
        payloadLen = Number(buffer.readBigUInt64BE(2));
        offset = 10;
      }

      const maskLen = masked ? 4 : 0;
      const totalLen = offset + maskLen + payloadLen;
      if (buffer.length < totalLen) return;

      const mask = masked ? buffer.slice(offset, offset + maskLen) : null;
      const payload = buffer.slice(offset + maskLen, totalLen);

      if (masked && mask) {
        for (let i = 0; i < payload.length; i++) {
          payload[i] ^= mask[i % 4];
        }
      }

      buffer = buffer.slice(totalLen);

      if (opcode === 0x08) {
        // Close frame
        closed = true;
        ws.readyState = 3;
        socket.end();
        (events['close'] || []).forEach(h => h());
        return;
      }

      if (opcode === 0x09) {
        // Ping → Pong
        const pong = Buffer.alloc(2);
        pong[0] = 0x8a;
        pong[1] = 0;
        socket.write(pong);
        continue;
      }

      if (opcode === 0x01 || opcode === 0x02) {
        const text = payload.toString('utf8');
        (events['message'] || []).forEach(h => h(text));
      }
    }
  });

  socket.on('close', () => {
    if (!closed) {
      closed = true;
      ws.readyState = 3;
      (events['close'] || []).forEach(h => h());
    }
  });

  socket.on('error', () => {
    if (!closed) {
      closed = true;
      ws.readyState = 3;
      (events['close'] || []).forEach(h => h());
    }
  });

  return ws;
}

// ─── HTTP Server ──────────────────────────────────────

const server = http.createServer((req, res) => {
  // Health check endpoint
  if (req.url === '/health') {
    const roomCount = rooms.size;
    let peerCount = 0;
    for (const room of rooms.values()) peerCount += room.peers.size;
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', rooms: roomCount, peers: peerCount }));
    return;
  }

  // Room info endpoint
  if (req.url === '/rooms') {
    const info = [];
    for (const [id, room] of rooms) {
      const peers = [];
      for (const [pid, peer] of room.peers) {
        peers.push({ peerId: pid, userName: peer.userName });
      }
      info.push({ room: id, peers, opLogSize: room.opLog.length });
    }
    res.writeHead(200, {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
    });
    res.end(JSON.stringify(info));
    return;
  }

  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('s1engine collaboration relay server\n\nWebSocket endpoint: ws://localhost:' + PORT + '/\n');
});

server.on('upgrade', (req, socket, head) => {
  acceptWebSocket(req, socket, head);
});

server.listen(PORT, () => {
  console.log(`s1engine relay server listening on ws://localhost:${PORT}`);
  console.log(`Health check: http://localhost:${PORT}/health`);
  console.log(`Room info:    http://localhost:${PORT}/rooms`);
});
