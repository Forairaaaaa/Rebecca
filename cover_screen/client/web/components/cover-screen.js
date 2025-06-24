const logger = require("./logger");
const path = require("path");
const zmq = require("zeromq");
const fs = require("fs");

let _screens = [];

function _loadFrameBuffers(dir) {
  logger.info(`load screen from ${dir}`);

  const files = fs.readdirSync(dir);
  for (const file of files) {
    const fullPath = path.join(dir, file);
    if (path.extname(file) === ".json") {
      try {
        const data = fs.readFileSync(fullPath, "utf-8");
        const json = JSON.parse(data);
        _screens.push(json);
      } catch (err) {
        console.error(`Failed to load ${file}:`, err);
      }
    }
  }

  console.log(_screens);
}

async function _createSockets() {
  for (const screen of _screens) {
    const zmq_port = `tcp://127.0.0.1:${screen.port}`;

    logger.info(`connect to ${zmq_port}`);
    screen.socket = new zmq.Request();
    await screen.socket.connect(zmq_port);

    screen.push = async (data) => {
      logger.debug(`push data: ${data.length} bytes`);
      await screen.socket.send(data);
      const [reponse] = await screen.socket.receive();
      logger.debug(`reponse: ${reponse.toString()}`);
    };
  }
}

async function connect(fbTempDir = "/tmp/cover_screen") {
  logger.info("connect cover screens");
  if (_screens.length > 0) {
    await close();
  }
  _loadFrameBuffers(fbTempDir);
  await _createSockets();
}

function getScreens() {
  return _screens;
}

async function stop() {
  logger.info("stop cover screen");
  for (const screen of _screens) {
    await screen.socket.close();
  }
  _screens = [];
}

module.exports = { connect, getScreens, stop };
