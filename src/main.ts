import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

let currentTimeout: any;

window.addEventListener("DOMContentLoaded", async () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  const unlisten = await listen("core://update", async (event: any) => {
    console.log(event);
    const {
      title,
      artist,
      thumbnail: {
        content_type,
        data,
        dominant_color: [r, g, b],
      },
    } = event.payload.sessions[0];

    const titleElement: HTMLDivElement | null = document.getElementById(
      "title"
    ) as HTMLDivElement;

    const artistElement: HTMLDivElement | null = document.getElementById(
      "artist"
    ) as HTMLDivElement;

    console.log(title);

    const albumContainer: HTMLDivElement | null = document.getElementById(
      "album-container"
    ) as HTMLDivElement;

    const albumCover: HTMLImageElement | null = document.getElementById(
      "album"
    ) as HTMLImageElement;
    const albumCoverBackground: any =
      document.getElementById("background-album");
    const content = new Uint8Array(data);

    const contentURL = URL.createObjectURL(
      new Blob([content.buffer], { type: content_type })
    );

    albumCover.src = contentURL;
    albumCoverBackground.src = contentURL;
    titleElement.textContent = title;
    artistElement.textContent = artist;

    const identifier = {};
    currentTimeout = identifier;

    await appWindow.show();

    setTimeout(async () => {
      if (identifier === currentTimeout) {
        await appWindow.hide();
      }
    }, 10000);
  });
});
