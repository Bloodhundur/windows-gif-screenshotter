
let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

import tauriapi from '@tauri-apps/api';
const { invoke } = tauriapi.tauri;
import { listen } from '@tauri-apps/api/event';

listen('message-from-rust', (event) => {
  console.log('Received from Rust:', event.payload);
});

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  let startX, startY, endX, endY;

  document.addEventListener("mousedown", (event) => {
    startX = event.clientX;
    startY = event.clientY;
  });

  document.addEventListener("mouseup", (event) => {
    endX = event.clientX;
    endY = event.clientY;
    console.log(startX.startY, endX, endY)
  });

  const invoke = window.__TAURI__.core.invoke;

  async function getMouse() {
    const pos = await invoke("get_mouse_position");
    console.log("Mouse:", pos);
  }


  getMouse(); // Call it

});