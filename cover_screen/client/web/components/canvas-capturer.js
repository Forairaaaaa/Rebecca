const puppeteer = require("puppeteer");
const logger = require("./logger");

let browser = null;
let page = null;

async function start(port = 3000) {
  logger.info("start canvas capturer");
  browser = await puppeteer.launch({ headless: "new" });
  page = await browser.newPage();
  await page.goto(`http://localhost:${port}`);
}

async function capture(canvasId) {
  return await page.evaluate((id) => {
    const canvas = document.getElementById(id);
    if (!canvas) {
      return null;
    }
    const ctx = canvas.getContext("2d");
    const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    return Array.from(imgData.data);
  }, canvasId);
}

async function close() {
  if (browser) {
    logger.info("close canvas capturer");
    await browser.close();
    browser = null;
    page = null;
  }
}

module.exports = { start, capture, close };
