import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import "./styles.css";

import React from "react";
import { MantineProvider, localStorageColorSchemeManager } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import { createRoot } from "react-dom/client";

import App from "./App";
import { theme } from "./theme";

const colorSchemeManager = localStorageColorSchemeManager({
  key: "idp-color-scheme"
});

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Root element was not found");
}

createRoot(rootElement).render(
  <React.StrictMode>
    <MantineProvider
      theme={theme}
      defaultColorScheme="auto"
      colorSchemeManager={colorSchemeManager}
    >
      <Notifications position="top-right" />
      <App />
    </MantineProvider>
  </React.StrictMode>
);
