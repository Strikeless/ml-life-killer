<script lang="ts">
    let {
        show_while = true,
        show_after_ms = 500,
    }: {
        show_while?: boolean,
        show_after_ms?: number,
    } = $props();

    const rectangles = [
        { animation_delay_multiple: 0 },
        { animation_delay_multiple: 1 },
        { animation_delay_multiple: 2 },

        { animation_delay_multiple: 1 },
        { animation_delay_multiple: 2 },
        { animation_delay_multiple: 3 },

        { animation_delay_multiple: 2 },
        { animation_delay_multiple: 3 },
        { animation_delay_multiple: 4 },
    ];
</script>

<style lang="less">
    @import "../global.less";

    .loading-throbber-container {
        position: absolute;
        width: max-content;
        height: max-content;
        margin: auto;
    }

    .loading-throbber {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        width: 3rem;
        height: 3rem;
        gap: 0.5rem;
    }

    .loading-throbber__rectangle {
        display: flex;
        width: 100%;
        height: 100%;
        background-color: #F00;
        background-color: @color-fg;
        border-radius: calc(1rem / 10);

        transform: scale(0.0);
        animation: loading-throbber-rectangle-animation cubic-bezier(0.5, 0.0, 0.5, 1.0) 2000ms infinite;
        animation-delay: calc(300ms * var(--animation-delay-multiple) + var(--show-after));
    }
    @keyframes loading-throbber-rectangle-animation {
        0%, 100% {
            transform: scale(0.0);
        }

        50% {
            transform: scale(1.0);
        }
    }
</style>

{#if show_while}
<div class="loading-throbber-container">
    <div class="loading-throbber" style="--show-after: {show_after_ms}ms;">
        {#each rectangles as rectangle}
        <div class="loading-throbber__rectangle" style="--animation-delay-multiple: {rectangle.animation_delay_multiple};"></div>
        {/each}
    </div>
</div>
{/if}
