import { createServer } from "http";
import { parse } from "url";
import next from "next";
import { IncomingMessage, ServerResponse } from "http";
import { NextUrlWithParsedQuery } from "next/dist/server/request-meta";

const port = parseInt(process.env.PORT || "3000", 10);
const dev = process.env.NODE_ENV !== "production";
const app = next({ dev });
const handle = app.getRequestHandler();

app.prepare().then(() => {
  createServer((req, res) => {
    const parsedUrl = parse(req.url!, true);

    if (myApiRoutes(req, res, parsedUrl)) {
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

function myApiRoutes(
  req: IncomingMessage,
  res: ServerResponse,
  parsedUrl?: NextUrlWithParsedQuery | undefined
) {
  const { pathname } = parsedUrl!;

  if (pathname === "/api/hello") {
    res.statusCode = 200;
    res.setHeader("Content-Type", "application/json");
    res.end(JSON.stringify({ message: "Hello from custom server API!" }));
    return true;
  }

  return false;
}
