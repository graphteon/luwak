import { Webview } from "https://github.com/webview/webview_deno/blob/main/mod.ts";

const html = `
  <html>
  <body>
    <h1>Hello from Luwak v${Luwak.version}</h1>
  </body>
  </html>
`;

const webview = new Webview();

webview.navigate(`data:text/html,${encodeURIComponent(html)}`);
webview.run();