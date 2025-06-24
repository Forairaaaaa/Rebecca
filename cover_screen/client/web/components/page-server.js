const WebSocket = require("ws");
const express = require("express");
const logger = require("./logger");

let _server = null;
let _wss = null;
let _onRefresh = (canvasId) => {
  logger.warn("empty onRefresh callback");
};

function _handleBridgeMsg(ws, message) {
  try {
    const msg = JSON.parse(message);
    // console.log(msg);

    // Handle canvas refresh notification
    if (msg.action === "refresh" && msg.canvasId) {
      _onRefresh(msg.canvasId);
    }
  } catch (err) {
    logger.error(`handle msg error: ${err}`);
  }
}

function start(htmlDir, port = 3000) {
  logger.info("start page server");
  stop();

  // Create web server
  const app = express();
  app.use(express.static(htmlDir));
  _server = app.listen(port, () => {
    logger.info(`serving ${htmlDir} at http://localhost:${port}`);
  });

  // Create websocket for msg bridge
  _wss = new WebSocket.Server({ noServer: true });
  _server.on("upgrade", (request, socket, head) => {
    if (request.url === "/ws") {
      _wss.handleUpgrade(request, socket, head, (ws) => {
        _wss.emit("connection", ws, request);
      });
    } else {
      socket.destroy();
    }
  });

  _wss.on("connection", (ws) => {
    logger.info("client connected");
    ws.on("message", (message) => {
      _handleBridgeMsg(ws, message);
    });
  });
}

function stop() {
  if (_server) {
    logger.info("stop page server");
    _server.close();
    _server = null;
    _wss = null;
  }
}

/**
 * @param {function(string): void} callback
 */
function onRefresh(callback) {
  _onRefresh = callback;
}

module.exports = { start, stop, onRefresh };
