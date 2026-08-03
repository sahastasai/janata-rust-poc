# Web and mobile targets

## Web

Dioxus compiles the client to WebAssembly and renders browser DOM elements.
Cloudflare serves the static bundle globally and runs `/api/*` through the Rust
Worker. SPA fallback makes direct deep links resolve to the application shell.

## Android and iOS

Dioxus mobile runs Rust around the operating system's WebView. It is a native
application package, but it does not use native Android/iOS widgets for every
control. The shared CSS must therefore be tested in each platform WebView.

Native capabilities live behind narrow adapters:

- secure credential storage
- location and permission prompts
- maps and map interaction
- camera/photo selection and image processing
- push permission, token registration, and local notification
- deep links and app links
- share sheet and lifecycle restoration

Android builds require the SDK, NDK, emulator/device, and signing configuration.
iOS builds require macOS, Xcode, a simulator/device, and Apple signing for
device distribution. This Linux environment cannot honestly produce or verify
an iOS artifact; macOS CI is the required evidence path.
