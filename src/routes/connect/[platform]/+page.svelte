<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import { page } from "$app/stores";
    import { Loader2Icon } from "lucide-svelte";

    onMount(async () => {
        const window = getCurrentWindow();
        const platform = $page.params.platform;
        const token = await invoke("get_platform_session", {
            platform,
        });
        await window.emit("connection-callback", { token });
        window.close();
    });
</script>

<main
    class="h-[100dvh] gap-4 w-screen items-center justify-start flex flex-col bg-background"
>
    <h1>Connecting to {$page.params.platform}...</h1>
    <Loader2Icon class="h-4 w-4 animate-spin" />
</main>
