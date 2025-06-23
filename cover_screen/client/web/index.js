const puppeteer = require("puppeteer");
const path = require("path");
const coverScreen = require("./components/cover-screen");
const webServer = require("./components/web-server");
const logger = require("./components/logger");

const HTML_DIR = path.join(__dirname, "page");
const PORT = 3000; // 本地服务器端口
const CANVAS_ID = "cover-screen-0"; // 要获取的 canvas ID
const INTERVAL = 200; // 毫秒间隔

(async () => {
  await coverScreen.connect();
  logger.info(`screen num: ${coverScreen.getScreens().length}`);

  webServer.start(HTML_DIR, PORT);

  // 启动 Puppeteer 无头浏览器
  const browser = await puppeteer.launch({ headless: "new" });
  const page = await browser.newPage();
  await page.goto(`http://localhost:${PORT}`);

  // 周期获取 Canvas 像素数据并发送
  setInterval(async () => {
    try {
      const rgba = await page.evaluate((id) => {
        const canvas = document.getElementById(id);
        if (!canvas) return null;
        const ctx = canvas.getContext("2d");
        const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
        return Array.from(imgData.data); // 转为普通数组以便传输
      }, CANVAS_ID);

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
    console.log("Shutting down...");
    clearInterval();
    await browser.close();
    await coverScreen.close();
    webServer.stop();
    process.exit(0);
  });
})();
