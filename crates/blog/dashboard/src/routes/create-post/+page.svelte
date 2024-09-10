<script lang="ts">
    // import Quill from 'quill';
    // import EditorJs from '@editorjs/editorjs';
    import { onMount } from 'svelte';

    // onMount(() => {
    //     const quill = new Quill('#editor');

    //     // quill;
    // });
</script>

<!-- Sidebar -->
<!-- Add: (Media) Image, AI Image, Gallery, Video, Gif, File, (Elements) Divider, Button, Table, Expandable List, Poll, (From Web) HTML Code, SoundCloud, AdSense, (other) Products, Services,  -->
<!-- Settings: (general) Featured Image, Publish Date, Writer, Excerpt, Related Posts, Feature Post?, Comments?, (categories), (tags) -->
<!-- SEO: -->
<!-- Monetize: -->
<!-- Translate: -->
<!-- AI Tools: Create Post, Create Title, Create an Outline, Create Images, Add Meta Tags -->

<div class="w-full h-full bg-zinc-900 py-7 px-3">
    <!-- Toolbar -->
    <!-- Text Style (P, H1, H2), Text Size, Bold, Italicize, Underline, Text Color, Highlight Color, Quote, Code Snippet, Numbered List, Bulleted List, Alignment, Line Space, Dec/Inc Indent, Link,  -->
    <div class="mx-auto max-w-[940px] space-y-3">
        <div>
            <!-- Title -->
            <input
                class="variant-ghost w-full h-14 text-xl text-slate-50"
                placeholder="Add a Catchy Title"
                type="text"
            />
        </div>

        <div id="editor">
            <!-- Body -->
        </div>
    </div>
</div>
