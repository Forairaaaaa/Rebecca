const canvasCapturer = require("./components/canvas-capturer");
const coverScreen = require("./components/cover-screen");
const webServer = require("./components/page-server");
const logger = require("./components/logger");
const path = require("path");

const HTML_DIR = path.join(__dirname, "page");
const PORT = 3000; // 本地服务器端口
const CANVAS_ID = "cover-screen-0"; // 要获取的 canvas ID
const INTERVAL = 200; // 毫秒间隔

(async () => {
  await coverScreen.connect();
  webServer.start(HTML_DIR, PORT);
  await canvasCapturer.start(PORT);

  // 周期获取 Canvas 像素数据并发送
  setInterval(async () => {
    try {
      const rgba = await canvasCapturer.capture(CANVAS_ID);

      if (rgba) {
        for (const screen of coverScreen.getScreens()) {
          await screen.push(Buffer.from(rgba));
        }
      } else {
        console.warn("Canvas not found");
      }
    } catch (err) {
      console.error("Error capturing canvas:", err);
    }
  }, INTERVAL);

  // 捕捉退出信号
  process.on("SIGINT", async () => {
    logger.info("shutting down...");
    clearInterval();
    await canvasCapturer.close();
    await coverScreen.close();
    webServer.stop();
    process.exit(0);
  });
})();
