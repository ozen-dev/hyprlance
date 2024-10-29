<script lang="ts">
    import * as Table from "$lib/components/ui/table";
    import Logo from "$lib/assets/logo.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";
    import { PlusIcon, Loader2Icon, RotateCcwIcon } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { invoke } from "@tauri-apps/api/core";

    interface AuthData {
        email: string;
        uid: string;
        token: string;
    }

    let platforms = $state([
        {
            id: "contra",
            name: "Contra",
            connecting: false,
            auth: { email: "", uid: "", token: "" },
        },
        {
            id: "fiverr",
            name: "Fiverr",
            connecting: false,
            auth: { email: "", uid: "", token: "" },
        },
        {
            id: "upwork",
            name: "Upwork",
            connecting: false,
            auth: { email: "", uid: "", token: "" },
        },
    ]);

    function toggleLoadingState(platform: string) {
        platforms[platforms.findIndex((p) => p.id === platform)].connecting =
            !platforms[platforms.findIndex((p) => p.id === platform)]
                .connecting;
    }

    async function handleConnect(platform: string) {
        toggleLoadingState(platform);

        try {
            const data = (await invoke("authenticate", {
                platform,
            })) as AuthData;

            platforms.forEach((p) => {
                if (p.id === platform) {
                    p.auth = data;
                }
            });

            toast.success(`Connection established!`);
        } catch (err) {
            toast.error(String(err));
        }

        toggleLoadingState(platform);
    }
</script>

<main
    class="h-[100dvh] gap-8 w-screen items-center justify-start flex flex-col bg-gray-50"
>
    <nav class="flex w-full p-3 pt-20 justify-center items-center">
        <Logo></Logo>
    </nav>

    <h1 class="text-balance text-center max-w-[700px]">
        Welcome to the connection playground. From here, you can test the
        connection with different freelance platforms.
    </h1>

    <content class="mx-auto px-4 w-full max-w-[700px]">
        <section class="bg-background border rounded-md shadow-sm">
            <Table.Root class="w-full">
                <Table.Header>
                    <Table.Row>
                        <Table.Head class="font-normal w-32 text-sm">
                            Platform
                        </Table.Head>
                        <Table.Head class="font-normal w-52 text-sm">
                            Email
                        </Table.Head>
                        <Table.Head class="font-normal w-32 text-sm">
                            UID
                        </Table.Head>
                        <Table.Head class="font-normal w-32 text-sm">
                            Token
                        </Table.Head>
                        <Table.Head class="font-normal text-sm text-right"
                        ></Table.Head>
                    </Table.Row>
                </Table.Header>
                <Table.Body>
                    {#each platforms as p}
                        <Table.Row>
                            <Table.Cell class="font-medium w-32">
                                <div class="flex items-center gap-2">
                                    <img
                                        src={`/src/lib/assets/${p.id}-icon.png`}
                                        alt={p.name}
                                        class="w-5 h-5 rounded-full"
                                    />
                                    {p.name}
                                </div>
                            </Table.Cell>
                            <Table.Cell class="w-52">
                                {#if p.connecting}
                                    <Skeleton class="w-40 h-4" />
                                {:else}
                                    {p.auth.email}
                                {/if}
                            </Table.Cell>
                            <Table.Cell class="w-32">
                                {#if p.connecting}
                                    <Skeleton class="w-12 h-4" />
                                {:else}
                                    {p.auth.uid}
                                {/if}
                            </Table.Cell>
                            <Table.Cell class="w-32">
                                {#if p.connecting}
                                    <Skeleton class="w-12 h-4" />
                                {:else}
                                    {p.auth.token}
                                {/if}
                            </Table.Cell>
                            <Table.Cell class="text-right">
                                <Button
                                    variant="outline"
                                    size="icon"
                                    disabled={p.connecting}
                                    onclick={async () =>
                                        await handleConnect(p.id)}
                                >
                                    {#if p.auth.token}
                                        <RotateCcwIcon class="h-4 w-4" />
                                    {:else if p.connecting}
                                        <Loader2Icon
                                            class="h-4 w-4 animate-spin "
                                        />
                                    {:else}
                                        <PlusIcon class="h-4 w-4" />
                                    {/if}
                                </Button>
                            </Table.Cell>
                        </Table.Row>
                    {/each}
                </Table.Body>
            </Table.Root>
        </section>
    </content>

    <Button
        variant="outline"
        size="sm"
        onclick={() => window.location.reload()}
    >
        Reset all
    </Button>
</main>
