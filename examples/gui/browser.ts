import { Webview } from "https://github.com/webview/webview_deno/blob/main/mod.ts";

const webview = new Webview();
webview.navigate("https://google.com");
webview.run();