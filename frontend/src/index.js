import './i18n';
import App from "./App/";
import React from "react";
import { createRoot } from "react-dom/client";
import {
    useLocation,
    useNavigationType,
    createRoutesFromChildren,
    matchRoutes,
} from "react-router-dom";
import * as Sentry from "@sentry/react";
import { BrowserTracing } from "@sentry/tracing";

Sentry.init({
    dsn: "https://59c861559677953130945392096c08b6@sentry.suyou.org/8",
    release: RELEASE,
    integrations: [new BrowserTracing({
        routingInstrumentation: Sentry.reactRouterV6Instrumentation(
            React.useEffect,
            useLocation,
            useNavigationType,
            createRoutesFromChildren,
            matchRoutes,
        ),
    })],

    // We recommend adjusting this value in production, or using tracesSampler
    // for finer control
    tracesSampleRate: 1.0,
});

const container = document.getElementById("root");
const root = createRoot(container);
root.render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);

function $buo_f() {
    var e = document.createElement("script");
    e.src = "//browser-update.org/update.min.js";
    document.body.appendChild(e);
}
try {
    document.addEventListener("DOMContentLoaded", $buo_f, false);
} catch (e) {
    window.attachEvent("onload", $buo_f);
}
