用 page 里的静态编译创建创建一个 web server

page 在一个 canvas 上绘制，刷新时调用接口 `/api/refresh/:canvasId`，触发 puppeteer 截图转发到 socket

至少能用🤡

## build page

cd page

npm install

npm run build

cd ..

## run client

npm install

npm run dev
