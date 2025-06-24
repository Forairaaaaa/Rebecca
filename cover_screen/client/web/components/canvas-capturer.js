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
    return await _page.evaluate(async (id) => {
      const el = document.getElementById(id);
      if (!el) {
        throw new Error(`element ${id} not found`);
      }

      let canvas = null;

      // If element is canvas, use it directly
      if (el.tagName.toLowerCase() == "canvas") {
        canvas = el;
      }
      // If not, convert to canvas
      else {
        if (!window.html2canvas) {
          throw new Error("html2canvas is not loaded");
        }
        canvas = await html2canvas(el, { backgroundColor: null });
      }

      if (!canvas) {
        throw new Error(`get canvas ${id} error`);
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
    await _page.close();
    await _browser.close();
    _browser = null;
    _page = null;
  }
}

module.exports = { start, capture, stop };
