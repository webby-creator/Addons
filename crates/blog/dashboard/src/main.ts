import { mount } from 'svelte';

import App from './App.svelte';
import './app.pcss';

const innerElement = document.getElementById("embed-inner");

if (innerElement == null) {
    throw new Error("Unable to find Element to mount to.");
}

mount(App, { target: innerElement, props: { path } });

// TODO: Remove static path in dashboard
// TODO: unmount
// TODO: change path
// TODO: External Events (for path changes)