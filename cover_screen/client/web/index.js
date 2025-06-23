const zmq = require("zeromq");
const puppeteer = require("puppeteer");
const path = require("path");
const express = require("express");

const HTML_DIR = path.join(__dirname, "page");
const PORT = 3000; // 本地服务器端口
const ZMQ_TARGET = "tcp://127.0.0.1:50000"; // zmq 目标地址
const CANVAS_ID = "cover-screen-0"; // 要获取的 canvas ID
const INTERVAL = 200; // 毫秒间隔

(async () => {
    // 启动静态服务器提供 HTML 页面
    const app = express();
    app.use(express.static(HTML_DIR));
    const server = app.listen(PORT, () => {
        console.log(`Serving ${HTML_DIR} at http://localhost:${PORT}`);
    });

    // 初始化 ZMQ Socket
    const sock = new zmq.Request();
    await sock.connect(ZMQ_TARGET);
    console.log("ZMQ request connected to", ZMQ_TARGET);

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
                await sock.send(Buffer.from(rgba));
                // await sock.send("??????");
                console.log("Sent frame:", rgba.length, "bytes");
                // console.log(rgba);
                const [reponse] = await sock.receive();
                console.log("reponse:", reponse.toString());
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
        await sock.close();
        server.close();
        process.exit(0);
    });
})();
