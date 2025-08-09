const logger = require("./logger");
const zmq = require("zeromq");
const http = require("http");

let _screens = [];
const API_PORT = 12580;

function _httpRequest(hostname, port, path) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname,
      port,
      path,
      method: 'GET',
      headers: {
        'Content-Type': 'application/json'
      }
    };

    const req = http.request(options, (res) => {
      let data = '';
      
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        try {
          const json = JSON.parse(data);
          resolve(json);
        } catch (err) {
          reject(new Error(`Failed to parse JSON: ${err.message}`));
        }
      });
    });

    req.on('error', (err) => {
      reject(err);
    });

    req.end();
  });
}

async function _loadScreenInfos() {
  logger.info(`load screen info from HTTP API`);
  
  try {
    const devices = await _httpRequest('127.0.0.1', API_PORT, '/get-device/all');
    _screens = devices.map(device => ({
      id: device.id,
      ...device.info
    }));
    
    console.log(_screens);
  } catch (err) {
    console.error('Failed to load screen info from API:', err);
    throw err;
  }
}

async function _createSockets() {
  for (const screen of _screens) {
    const zmq_port = `tcp://127.0.0.1:${screen.frame_buffer_port}`;

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

async function connect() {
  logger.info("connect cover screens");
  if (_screens.length > 0) {
    await stop();
  }
  await _loadScreenInfos();
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
