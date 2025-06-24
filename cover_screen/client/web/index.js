const canvasCapturer = require("./components/canvas-capturer");
const coverScreen = require("./components/cover-screen");
const pageServer = require("./components/page-server");
const logger = require("./components/logger");
const path = require("path");

const HTML_DIR = path.join(__dirname, "page/out");
const PORT = 4321;

async function main() {
  logger.info("start cover screen web client");

  // Try connect cover screens
  await coverScreen.connect();
  logger.info(`screen num: ${coverScreen.getScreens().length}`);

  // Create screen canvas map
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
  logger.info("setup canvas refresh callback");
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

  await new Promise((resolve) => {
    process.on("SIGINT", resolve);
    process.on("SIGTERM", resolve);
    logger.info("client running...");
  });

  // Handle shutdown
  logger.info("shutting down...");
  await canvasCapturer.stop();
  pageServer.stop();
  await coverScreen.stop();
  process.exit(0);
}

main().catch((err) => {
  logger.error("client error:", err);
  process.exit(1);
});
