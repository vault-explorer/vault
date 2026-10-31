import { render } from "solid-js/web";

import { App } from "./App";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("#root element missing from index.html");

// Bench builds only: this branch and the chunk behind it don't exist in a normal
// build, because Vite replaces the variable at build time.
if (import.meta.env.VITE_VAULT_BENCH === "1" && location.search === "?bench=scroll") {
  void import("./bench/ScrollProbe").then(({ ScrollProbe }) => {
    render(() => <ScrollProbe />, root);
  });
} else {
  render(() => <App />, root);
}
