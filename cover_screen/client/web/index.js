const canvasCapturer = require("./components/canvas-capturer");
const coverScreen = require("./components/cover-screen");
const pageServer = require("./components/page-server");
const logger = require("./components/logger");
const path = require("path");

const HTML_DIR = path.join(__dirname, "page");
const PORT = 3000;

(async () => {
  await coverScreen.connect();

  pageServer.start(HTML_DIR, PORT);

  await canvasCapturer.start(PORT);

  pageServer.onRefresh(async (canvasId) => {
    const rgba = await canvasCapturer.capture(canvasId);
    if (rgba) {
      for (const screen of coverScreen.getScreens()) {
        await screen.push(Buffer.from(rgba));
      }
    } else {
      logger.error(`invalid canvas id ${canvasId} data`);
    }
  });

  process.on("SIGINT", async () => {
    logger.info("shutting down...");
    clearInterval();
    await canvasCapturer.stop();
    await coverScreen.stop();
    pageServer.stop();
    process.exit(0);
  });
})();
