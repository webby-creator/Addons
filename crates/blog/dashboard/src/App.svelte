<!-- <svelte:options customElement="dash-embed" /> -->

<script lang="ts">
    import type { Component } from "svelte";

    import rootRoute from "./routes/+page.svelte";
    import analyticsRoute from "./routes/analytics/+page.svelte";
    import categoriesRoute from "./routes/categories/+page.svelte";
    import commentsRoute from "./routes/comments/+page.svelte";
    import createPostRoute from "./routes/create-post/+page.svelte";
    import postsRoute from "./routes/posts/+page.svelte";
    import tagsRoute from "./routes/tags/+page.svelte";

    let { path }: { path: string } = $props();

    const routes: { [path: string]: Component } = {
        "/": rootRoute,
        "/analytics": analyticsRoute,
        "/categories": categoriesRoute,
        "/comments": commentsRoute,
        "/create-post": createPostRoute,
        "/posts": postsRoute,
        "/tags": tagsRoute,
    };

    let Route = routes[path];
</script>

<main>
    {#if Route != null}
        <Route />
    {/if}
</main>
