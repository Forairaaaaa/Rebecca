const canvasCapturer = require("./components/canvas-capturer");
const coverScreen = require("./components/cover-screen");
const pageServer = require("./components/page-server");
const logger = require("./components/logger");
const path = require("path");

const HTML_DIR = path.join(__dirname, "page");
const PORT = 4321;

(async () => {
  logger.info("start cover screen web client");

  // Try connect cover screens
  await coverScreen.connect();
  logger.info(`screen num: ${coverScreen.getScreens().length}`);

  // Create screen canvas ma
  logger.info("create screen canvas map");
  const screenCanvasMap = new Map();
  for (let i = 0; i < coverScreen.getScreens().length; i++) {
    screenCanvasMap.set(`cover-screen-${i}`, i);
  }
  console.log(screenCanvasMap);

  // Start page server
  pageServer.start(HTML_DIR, PORT);

  // Start canvas capturer
  await canvasCapturer.start(PORT);

  // Handle canvas refresh
  pageServer.onRefresh(async (canvasId) => {
    // Check map
    if (screenCanvasMap.has(canvasId)) {
      // Capture canvas rgba data
      const rgba = await canvasCapturer.capture(canvasId);
      if (rgba) {
        // Push to screen
        const screenIndex = screenCanvasMap.get(canvasId);
        await coverScreen.getScreens()[screenIndex].push(Buffer.from(rgba));
      } else {
        logger.error(`invalid canvas id ${canvasId} data`);
      }
    }
  });

  // Handle shutdown
  process.on("SIGINT", async () => {
    logger.info("shutting down...");
    clearInterval();
    await canvasCapturer.stop();
    await coverScreen.stop();
    pageServer.stop();
    process.exit(0);
  });
})();
