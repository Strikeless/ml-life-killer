<script lang="ts">
    import type { BoardInfoIdPair } from "../api/types";

    const {
        boards,
        from_index,
    }: {
        boards: BoardInfoIdPair[],
        from_index: number,
    } = $props();
</script>

<style>
    .board-list-page {
        display: flex;
        flex-direction: column;
    }

    .board-list-page__entry {
        display: flex;
        flex-direction: row;
    }

    .board-list-page__entry__size {
        display: flex;
        flex-direction: row;
    }
    .board-list-page__entry__size__axis + .board-list-page__entry__size__axis::before {
        content: "x";
    }
</style>

<div class="board-list-page">
    {#each boards as board, board_index}
    <a class="board-list-page__entry" href="#" data-board-id="{board.id}">
        <p class="board-list-page__entry__number">#{1 + from_index + board_index}</p>
        <p class="board-list-page__entry__title">{board.info.title}</p>
        <p class="board-list-page__entry__rules">{board.info.rules}</p>
        <div class="board-list-page__entry__size">
            <p class="board-list-page__entry__size__axis">{board.info.width}</p>
            <p class="board-list-page__entry__size__axis">{board.info.height}</p>
        </div>
    </a>
    {/each}
</div>
