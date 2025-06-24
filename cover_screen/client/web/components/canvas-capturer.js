const { createCanvas, loadImage } = require("canvas");
const puppeteer = require("puppeteer");
const logger = require("./logger");

let _browser = null;
let _page = null;

async function start(port = 3000) {
  logger.info("start canvas capturer");

  _browser = await puppeteer.launch({
    headless: "new",
    handleSIGINT: false,
    handleSIGHUP: false,
    handleSIGTERM: false,
  });
  _page = await _browser.newPage();

  await _page.goto(`http://localhost:${port}`);

  await _page.addScriptTag({
    url: "https://cdn.jsdelivr.net/npm/html2canvas-pro@1.5.11/dist/html2canvas-pro.min.js",
  });
}

/**
 * @param {string} canvasId
 * @returns {Promise<Uint8Array>} canvas RGBA data
 */
async function capture(canvasId) {
  try {
    // Take screenshot of the element
    const element = await _page.$(`#${canvasId}`);
    const buffer = await element.screenshot({
      type: "png",
    });
    // console.log(buffer);

    // Create canvas from the screenshot
    const image = await loadImage(buffer);
    const canvas = createCanvas(image.width, image.height);
    const ctx = canvas.getContext("2d");
    ctx.drawImage(image, 0, 0);

    // Return the canvas data
    const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    return Array.from(imgData.data);
  } catch (err) {
    logger.error(`capture ${canvasId} error: ${err}`);
    return null;
  }
}

async function stop() {
  if (_browser) {
    logger.info("stop canvas capturer");
    await _page.close();
    await _browser.close();
    _browser = null;
    _page = null;
  }
}

module.exports = { start, capture, stop };
