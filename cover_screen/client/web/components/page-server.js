const express = require("express");
const logger = require("./logger");

let server = null;

function start(htmlDir, port = 3000) {
  logger.info("start page server");
  stop();
  const app = express();
  app.use(express.static(htmlDir));
  server = app.listen(port, () => {
    logger.info(`serving ${htmlDir} at http://localhost:${port}`);
  });
}

function stop() {
  if (server) {
    logger.info("stop page server");
    server.close();
    server = null;
  }
}

module.exports = { start, stop };
