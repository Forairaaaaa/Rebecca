const puppeteer = require("puppeteer");
const logger = require("./logger");

let _browser = null;
let _page = null;

async function start(port = 3000) {
  logger.info("start canvas capturer");
  _browser = await puppeteer.launch({ headless: "new" });
  _page = await _browser.newPage();
  await _page.goto(`http://localhost:${port}`);
}

/**
 * @param {string} canvasId
 * @returns {Promise<Uint8Array>} canvas RGBA data
 */
async function capture(canvasId) {
  try {
    return await _page.evaluate((id) => {
      const canvas = document.getElementById(id);
      if (!canvas) {
        return null;
      }
      const ctx = canvas.getContext("2d");
      const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      return Array.from(imgData.data);
    }, canvasId);
  } catch (err) {
    logger.error(`capture canvas ${canvasId} error: ${err}`);
    return null;
  }
}

async function stop() {
  if (_browser) {
    logger.info("stop canvas capturer");
    await _browser.close();
    _browser = null;
    _page = null;
  }
}

module.exports = { start, capture, stop };
