const { listen } = window.__TAURI__.event

listen("resize_square", (eventpayload) => {
  print("hello");
});