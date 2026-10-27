import { mount } from "svelte";
import "./styles/tokens.css";
import "./styles/motion.css";
import "./styles/globals.css";
import "./styles/panels.css";
import "./styles/settings.css";
import App from "./app/App.svelte";

const appElement = document.getElementById("app");
if (!appElement) {
  throw new Error("Failed to find root element #app");
}

const app = mount(App, {
  target: appElement,
});

export default app;
