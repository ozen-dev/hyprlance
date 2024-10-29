<script lang="ts">
    import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
    import Logo from "$lib/assets/logo.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import Badge from "$lib/components/ui/badge/badge.svelte";
    import { RotateCcwIcon } from "lucide-svelte";

    const platforms = $state([
        { id: "contra", name: "Contra", token: "" },
        { id: "fiverr", name: "Fiverr", token: "" },
        { id: "upwork", name: "Upwork", token: "" },
        { id: "malt", name: "Malt", token: "" },
    ]);

    async function handleConnect(platform: string) {
        const webview = new WebviewWindow("auth", {
            url: `/connect/${platform}`,
            title: `Connect ${platform}`,
            width: 400,
            height: 600,
            resizable: false,
            center: true,
        });

        await webview.listen("connection-callback", (event) => {
            const { token } = event.payload as { token: string };
            platforms.forEach((p) => {
                if (p.id === platform) {
                    p.token = token;
                }
            });
        });
    }
</script>

<main
    class="h-[100dvh] gap-4 w-screen items-center justify-start flex flex-col bg-background"
>
    <nav class="flex mt-20 p-4 justify-center items-center">
        <Logo></Logo>
    </nav>

    <ul
        class="p-4 w-full max-w-[400px] grid grid-cols-1 gap-2 items-center justify-center"
    >
        {#each platforms as platform}
            <li
                class="group col-span-1 items-center gap-4 flex justify-between px-1 h-14 border-b"
            >
                <h1>
                    {platform.name}
                </h1>
                {#if !platform.token}
                    <Button
                        size="sm"
                        variant="secondary"
                        onclick={() => handleConnect(platform.id)}
                    >
                        Connect
                    </Button>
                {:else}
                    <Badge>
                        {platform.token}
                    </Badge>
                {/if}
            </li>
        {/each}
    </ul>

    <Button
        size="icon"
        variant="ghost"
        onclick={() => window.location.reload()}
    >
        <RotateCcwIcon class="h-4 w-4" />
    </Button>
</main>
