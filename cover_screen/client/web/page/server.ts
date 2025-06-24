import { createServer } from "http";
import { parse } from "url";
import next from "next";
import { IncomingMessage, ServerResponse } from "http";
import { NextUrlWithParsedQuery } from "next/dist/server/request-meta";
import si from "systeminformation";

const port = parseInt(process.env.PORT || "3000", 10);
const dev = process.env.NODE_ENV !== "production";
const app = next({ dev });
const handle = app.getRequestHandler();

app.prepare().then(() => {
  createServer(async (req, res) => {
    const parsedUrl = parse(req.url!, true);

    if (await myApiRoutes(req, res, parsedUrl)) {
      return;
    }

    handle(req, res, parsedUrl);
  }).listen(port);

  console.log(
    `> Server listening at http://localhost:${port} as ${
      dev ? "development" : process.env.NODE_ENV
    }`
  );
});

async function myApiRoutes(
  req: IncomingMessage,
  res: ServerResponse,
  parsedUrl?: NextUrlWithParsedQuery | undefined
) {
  const { pathname } = parsedUrl!;

  // Test
  if (pathname === "/api/hello") {
    res.statusCode = 200;
    res.setHeader("Content-Type", "application/json");
    res.end(JSON.stringify({ message: "Hello from custom server API!" }));
    return true;
  }

  // Get system info
  if (pathname === "/api/get-system-info" && req.method === "GET") {
    try {
      const system = await si.system();
      const cpu = await si.cpu();
      const mem = await si.mem();
      const osInfo = await si.osInfo();
      const currentLoad = await si.currentLoad();

      res.statusCode = 200;
      res.setHeader("Content-Type", "application/json");
      res.end(
        JSON.stringify({
          system,
          cpu,
          currentLoad,
          memory: {
            total: mem.total,
            free: mem.free,
            used: mem.used,
          },
          os: osInfo,
        })
      );
      return true;
    } catch (error) {
      res.statusCode = 500;
      res.setHeader("Content-Type", "application/json");
      res.end(
        JSON.stringify({
          error: "Failed to get system info",
          details: (error as Error).message,
        })
      );
    }
    return false;
  }

  const match = pathname?.match(/^\/api\/refresh\/([^/]+)$/);
  if (match) {
    // const value = match[1];
    // console.log("refresh", value);
    res.statusCode = 200;
    res.setHeader("Content-Type", "application/json");
    res.end(JSON.stringify({ message: "👌" }));
    return true;
  }

  return false;
}
