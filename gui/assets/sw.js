// Copyright (c) 2023 - 2025 Chair for Design Automation, TUM
// Copyright (c) 2025 Munich Quantum Software Company GmbH
// All rights reserved.
//
// SPDX-License-Identifier: MIT
//
// Licensed under the MIT License

const cacheName = "naviz-pwa";
const filesToCache = [
  "./",
  "./index.html",
  "./naviz-gui.js",
  "./naviz-gui_bg.wasm",
];

/* Start the service worker and cache all of the app's content */
self.addEventListener("install", (e) =>
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    }),
  ),
);

/* Serve cached content when offline */
self.addEventListener("fetch", (e) =>
  e.respondWith(
    caches.match(e.request).then(function (response) {
      return response || fetch(e.request);
    }),
  ),
);
