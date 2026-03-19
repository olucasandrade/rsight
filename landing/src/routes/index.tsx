import { component$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { LandingPage } from "../components/landing-page/landing-page";

export default component$(() => {
  return <LandingPage />;
});

export const head: DocumentHead = {
  title: "rsight — find anything in $HOME",
  meta: [
    {
      name: "description",
      content:
        "rsight is an open-source TUI written in Rust. Search files, folders, code content, and AI conversations locally in under a second. cargo install rsight.",
    },
  ],
};
