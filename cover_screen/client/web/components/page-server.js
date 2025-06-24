const express = require("express");
const logger = require("./logger");

let _server = null;
let _onRefresh = null;

function _handleApiRefresh(req, res) {
  const canvasId = req.params.canvasId;
  if (_onRefresh) {
    _onRefresh(canvasId);
  }
  res.json({ message: "👌" });
}

async function start(htmlDir, port = 3000) {
  logger.info("start page server");
  await stop();

  return new Promise((resolve, reject) => {
    // Create web server
    const app = express();
    app.use(express.static(htmlDir));

    app.get("/api/refresh/:canvasId", _handleApiRefresh);

    _server = app.listen(port, () => {
      logger.info(`serving ${htmlDir} at http://localhost:${port}`);
      resolve();
    });

    _server.on("error", reject);
  });
}

async function stop() {
  return new Promise((resolve, reject) => {
    if (_server) {
      logger.info("stop page server");
      _server.close((err) => {
        if (err) {
          logger.error(`stop page server error: ${err}`);
          reject(err);
        } else {
          _server = null;
          resolve();
        }
      });
    } else {
      resolve();
    }
  });
}

/**
 * @param {function(string): void} callback
 */
function onRefresh(callback) {
  _onRefresh = callback;
}

module.exports = { start, stop, onRefresh };
