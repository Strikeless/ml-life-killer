<script lang="ts">
    import { onMount } from "svelte";
    import { api } from "../../main";
    import type { HttpBoardListResponse } from "../api/types";
    import LoadingThrobber from "../LoadingThrobber.svelte";
    import BoardListPage from "./BoardListPage.svelte";

    let current_page_response: HttpBoardListResponse | undefined | null = $state();

    async function load_next_page(next_page_from_index?: number) {
        if (!next_page_from_index) {
            next_page_from_index = current_page_response != null
                ? current_page_response.from_index + current_page_response.boards.length
                : 0;
        }

        current_page_response = null; // Reset current page to show that we're loading the next one.
        current_page_response = await api.board_list({ from_index: next_page_from_index });
    }

    function on_clicked_create() {

    }

    onMount(() => { load_next_page(); });
</script>

<style>
    .board-list {
        display: flex;
        flex-direction: column;
    }

    .board-list__navbar {
        display: flex;
        flex-direction: row;
    }
</style>

<div class="board-list">
    <LoadingThrobber show_while={ !current_page_response } />

    {#if current_page_response}
    <div class="board-list__navbar">
        <p class="board-list__navbar__range">Listing pages #{1 + current_page_response.from_index}-{current_page_response.from_index + current_page_response.boards.length}</p>
        
        <button class="board-list__navbar__button board-list__navbar__button--first" disabled={current_page_response.from_index == 0} onclick={() => { load_next_page(0); }}>First</button>
        <button class="board-list__navbar__button board-list__navbar__button--next" disabled={!current_page_response.has_more} onclick={() => { load_next_page(); }}>Next</button>

        <button class="board-list__navbar__button board-list__navbar__button--create" onclick={on_clicked_create}>Create</button>
    </div>

    <BoardListPage boards={current_page_response.boards} from_index={current_page_response.from_index} />
    {/if}
</div>
