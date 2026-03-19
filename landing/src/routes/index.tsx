import { component$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { LandingPage } from "../components/landing-page/landing-page";

export default component$(() => {
  return <LandingPage />;
});

export const head: DocumentHead = {
  title: "rsight — find anything $HOME directory",
  meta: [
    {
      name: "description",
      content:
        "rsight is an open-source TUI written in Rust. Search files, folders, code content, and AI conversations in your $HOME directory in under a second. cargo install rsight.",
    },
    { property: "og:title", content: "rsight — find anything $HOME directory" },
    {
      property: "og:description",
      content:
        "rsight is an open-source TUI written in Rust. Search files, folders, code content, and AI conversations in your $HOME directory in under a second.",
    },
    { property: "og:image", content: "/logo.png" },
    { property: "og:type", content: "website" },
    { name: "twitter:card", content: "summary_large_image" },
    { name: "twitter:image", content: "/logo.png" },
  ],
};
